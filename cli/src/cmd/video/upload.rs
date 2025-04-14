use clap::Parser;
use indicatif::{MultiProgress, ProgressBar, ProgressDrawTarget, ProgressStyle};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{from_reader, to_writer};
use std::{
    cmp::min,
    fs::{self, File},
    io::{self, BufReader, Read, Seek, SeekFrom},
    path::{Path, PathBuf},
    sync::Mutex,
};
use tabled::{
    Table,
    settings::{Remove, Style, location::ByColumnName},
};

use crate::helpers::s3;
use crate::{api, helpers::cryptography::restricted_hash};

#[derive(Parser)]
pub(super) struct Args {
    #[arg(short = 'P', long)]
    no_progress: bool,
    #[arg(
        short = 's',
        long,
        default_value_t = 10_000_000,
        conflicts_with = "resume"
    )]
    part_size: u64,
    #[arg(short = 'R', long, default_value_t = 10)]
    retry_part: u64,
    #[arg(short, long)]
    password: Option<String>,
    #[arg(short, long)]
    thread_count: Option<usize>,
    #[arg(short, long)]
    resume: bool,

    uuid: String,
    path: PathBuf,
}

fn print_video_info(i: &api::video::Video) {
    let mut table = Table::new(&[i]);
    table.with(Style::modern());
    table.with(Remove::column(ByColumnName::new("Cover URL")));

    println!("{table}");
}

struct UploadMultiProgress {
    mp: MultiProgress,
    pb_parts: ProgressBar,
    pb_total: ProgressBar,
}

struct UploadProgress {
    mp: MultiProgress,
    pb_parts: ProgressBar,
    pb_total: ProgressBar,
    pb_current: ProgressBar,
    part_size: u64,
}

struct UploadProgressReadWrapper<R: Read> {
    read: R,
    pb_total: ProgressBar,
    pb_current: ProgressBar,
}

#[derive(Serialize, Deserialize)]
struct UploadStateData {
    upload_id: String,
    urls: Vec<String>,
    part_size: u64,
    etags: Vec<Option<String>>,
}

struct UploadState {
    pub upload_id: String,
    pub urls: Vec<String>,

    part_size: u64,
    etags: Mutex<Vec<Option<String>>>,
    state_file: PathBuf,
}

impl UploadState {
    fn to_data(&self) -> UploadStateData {
        let etags = self.etags.lock().unwrap().clone();

        UploadStateData {
            upload_id: self.upload_id.clone(),
            urls: self.urls.clone(),
            part_size: self.part_size,
            etags,
        }
    }

    fn from_data(data: UploadStateData, state_file: PathBuf) -> Self {
        if data.urls.len() != data.etags.len() {
            panic!("Size of URLs does not match size of finished array");
        }

        let upload_id = data.upload_id;
        let urls = data.urls;
        let part_size = data.part_size;
        let etags = Mutex::new(data.etags);
        Self {
            upload_id,
            urls,
            part_size,
            etags,
            state_file,
        }
    }

    fn write_state_file(&self) -> Result<(), Box<dyn std::error::Error>> {
        let f = File::create(self.state_file.as_path())?;
        let data = self.to_data();
        to_writer(f, &data)?;
        Ok(())
    }

    fn new<P: AsRef<Path>>(
        upload_id: String,
        urls: Vec<String>,
        part_size: u64,
        state_file: P,
    ) -> Self {
        let etags: Mutex<Vec<Option<String>>> = Mutex::new(vec![None; urls.len()]);
        Self {
            upload_id,
            urls,
            part_size,
            etags,
            state_file: state_file.as_ref().to_path_buf(),
        }
    }

    fn restore_from<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = File::open(path.as_ref())?;
        let data: UploadStateData = from_reader(file)?;
        Ok(Self::from_data(data, path.as_ref().to_path_buf()))
    }

    fn part_size(&self) -> u64 {
        self.part_size
    }

    fn is_finished(&self, i: usize) -> bool {
        self.etags.lock().unwrap()[i].is_some()
    }

    fn set_etag(&self, i: usize, etag: String) {
        self.etags.lock().unwrap()[i] = Some(etag)
    }

    fn collect_etags(&self) -> Vec<String> {
        self.etags
            .lock()
            .unwrap()
            .iter()
            .map(|v| {
                v.as_ref()
                    .expect("All ETags must be set before calling collect_etags()")
                    .to_string()
            })
            .collect()
    }
}

impl UploadMultiProgress {
    fn new(parts: u64, total: u64) -> Self {
        let mp = MultiProgress::new();

        let pb_parts = mp
            .add(ProgressBar::new(parts))
            .with_prefix("Parts")
            .with_style(
                ProgressStyle::default_bar()
                    .template("{prefix:8} {wide_bar:40.cyan/blue} {pos}/{len}")
                    .unwrap()
                    .progress_chars("##-"),
            );
        pb_parts.set_position(0);

        let pb_total = mp
            .add(ProgressBar::new(total))
            .with_prefix("Total")
            .with_style(
                ProgressStyle::default_bar()
                    .template(
                        "{prefix:8} {wide_bar:40.green/black} {bytes}/{total_bytes} \
			       {bytes_per_sec} elapsed {elapsed} ETA {eta}",
                    )
                    .unwrap(),
            );

        Self {
            mp,
            pb_parts,
            pb_total,
        }
    }

    fn new_part(&self, part_idx: usize, part_size: u64) -> UploadProgress {
        let pb_current = self
            .mp
            .add(ProgressBar::new(part_size))
            .with_prefix(format!("#{}", part_idx + 1))
            .with_style(
                ProgressStyle::default_bar()
                    .template(
                        "{prefix:8} {wide_bar:40.yellow/black} {bytes}/{total_bytes} \
			       {bytes_per_sec}",
                    )
                    .unwrap(),
            )
            .with_position(0);

        UploadProgress {
            mp: self.mp.clone(),
            pb_parts: self.pb_parts.clone(),
            pb_total: self.pb_total.clone(),
            pb_current,
            part_size,
        }
    }

    fn hide(&self) {
        self.mp.set_draw_target(ProgressDrawTarget::hidden())
    }

    fn println<S: AsRef<str>>(&self, msg: S) -> io::Result<()> {
        if self.mp.is_hidden() {
            println!("{}", msg.as_ref());
            Ok(())
        } else {
            self.mp.println(msg)
        }
    }

    fn finish(&self) {
        self.pb_parts.finish();
        self.pb_total.finish();
    }
}

impl UploadProgress {
    fn wrap_read<R: Read>(&self, read: R) -> UploadProgressReadWrapper<R> {
        UploadProgressReadWrapper {
            read,
            pb_total: self.pb_total.clone(),
            pb_current: self.pb_current.clone(),
        }
    }

    fn finish(&self) {
        self.pb_current.finish();
        self.pb_parts.inc(1);
        self.mp.remove(&self.pb_current);
    }

    fn skip(&self) {
        self.pb_total.inc(self.part_size);
        self.finish()
    }

    fn reset(&self) {
        self.pb_current.reset();
    }
}

impl<R: Read> Read for UploadProgressReadWrapper<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let bytes_read = self.read.read(buf)?;
        self.pb_current.inc(bytes_read as u64);
        self.pb_total.inc(bytes_read as u64);
        Ok(bytes_read)
    }
}

fn do_upload_part(
    uploader: &s3::Uploader,
    path: &Path,
    url: &str,
    offset: u64,
    size: u64,
    pb: &UploadProgress,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut f = File::open(path)?;
    f.seek(SeekFrom::Start(offset))?;

    let part_reader = f.take(size);
    let buf_reader = BufReader::new(part_reader);

    let req = uploader
        .url(url)
        .mimetype("video/mp4")
        .from_reader_sized(pb.wrap_read(buf_reader), size);

    let res = req.upload()?;

    Ok(res.etag)
}

fn upload_part(
    uploader: &s3::Uploader,
    path: &Path,
    url: &str,
    offset: u64,
    size: u64,
    pb: &UploadProgress,
    retry: u64,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut retry_count = 0;
    loop {
        let result = do_upload_part(uploader, path, url, offset, size, &pb);
        if result.is_ok() {
            pb.finish();
            return result;
        }

        retry_count += 1;
        if retry_count >= retry {
            return result;
        }
        pb.reset();
    }
}

fn do_upload(
    uuid: &str,
    path: &Path,
    part_size: u64,
    no_progress: bool,
    retry: u64,
    hash: Option<String>,
    resume: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let f = File::open(path)?;
    let f_size = f.metadata()?.len();

    let state_file_path = path.to_path_buf().with_extension("progress");
    let state = if resume {
        UploadState::restore_from(state_file_path.as_path())?
    } else {
        if fs::exists(state_file_path.as_path())? {
            eprintln!("Upload progress file exists; cowardly refuse to start new upload.");
            eprintln!(
                "Pass --resume to pick up previous progress, \
		       or remove {} if you would like to start fresh.",
                state_file_path.display()
            );
            std::process::exit(-1)
        }

        let upload_start = api::video::upload_start(uuid, f_size, part_size, hash.clone())?;
        print_video_info(&upload_start.video);
        UploadState::new(
            upload_start.upload_id,
            upload_start.urls,
            part_size,
            state_file_path.as_path(),
        )
    };
    state.write_state_file()?;

    std::println!("Initiating multi-part upload");

    let part_size = state.part_size();
    let parts = state.urls.len() as u64;

    let mp = UploadMultiProgress::new(parts, f_size);
    if no_progress {
        mp.hide();
    }

    let uploader = s3::Uploader::new()?;

    state.urls.par_iter().enumerate().for_each(|(i, url)| {
        let offset = (i as u64) * part_size;
        let size = min(part_size, f_size - offset);
        let pb = mp.new_part(i, size);

        if state.is_finished(i) {
            mp.println(format!(
                "Skipping part {} since it's already finished",
                i + 1
            ))
            .unwrap();
            pb.skip();
        } else {
            let etag = upload_part(&uploader, path, url, offset, size, &pb, retry)
                .expect(format!("Failed to uploading part {}", i + 1).as_str());
            state.set_etag(i, etag);
            state
                .write_state_file()
                .expect("Failed to write state file");
            mp.println(format!("Part {} uploaded", i + 1)).unwrap();
        }
    });

    let etags = state.collect_etags();
    api::video::upload_finish(uuid, state.upload_id, etags, hash)?;
    mp.finish();
    fs::remove_file(state_file_path)?;

    Ok(())
}

pub(crate) fn main(args: Args) {
    std::println!("Uploading video file {path}", path = args.path.display());

    args.thread_count.map(|tc| {
        println!("Setting thread count to {tc}");
        rayon::ThreadPoolBuilder::new()
            .num_threads(tc)
            .build_global()
            .expect("Failed to set thread count")
    });

    let hash = args
        .password
        .map(|v| restricted_hash(&args.uuid, &v).unwrap());

    do_upload(
        &args.uuid,
        &args.path,
        args.part_size,
        args.no_progress,
        args.retry_part,
        hash,
        args.resume,
    )
    .unwrap();
    println!("Upload finished")
}
