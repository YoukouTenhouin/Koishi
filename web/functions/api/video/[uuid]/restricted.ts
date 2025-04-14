import { AwsClient } from 'aws4fetch'
import * as v from 'valibot'

import { obj_urls } from '@flib/objects'
import { video_by_uuid_with_hash, run_query } from '@flib/queries'
import { get_req_body } from '@flib/requests'
import { res } from '@flib/responses'
import { Env } from '@flib/types'

export const onRequestGet: PagesFunction<Env> = async (context) => {
    const uuid = context.params.uuid as string

    const { success, video, error } = await video_by_uuid_with_hash(context.env.DB, uuid)
    if (!success) {
        return res.db_transaction_error(error)
    }
    if (!video) {
        return res.not_found(`Video ${uuid} not found`)
    }

    const q_params = new URL(context.request.url).searchParams
    const verify_hash = q_params.get("verify_hash")?.toLowerCase()

    if (!video.restricted) {
        return res.ok({
            restricted: video.restricted
        })
    } else {
        return res.ok({
            restricted: video.restricted,
            hash_verified: verify_hash ? verify_hash == video.restricted_hash : undefined
        })
    }
}

const PostBody = v.variant('command', [
    v.object({
        command: v.literal("copy_start"),
        copy_source: v.string(),
        part_size: v.number(),
        hash: v.nullish(v.string()),
    }),
    v.object({
        command: v.literal("copy_finish"),
        copy_source: v.string(),
        etags: v.array(v.string()),
        upload_id: v.string(),
        hash: v.nullish(v.string()),
    }),
])

async function get_s3_url_info(aws: AwsClient, url: string): Promise<[boolean, number | null]> {
    const headRes = await aws.fetch(url, { method: "HEAD" })
    if (headRes.status === 404) {
        return [false, null]
    } else if (!headRes.ok) {
        throw Error(`Status { headRes.status } while requesting from s3`)
    }

    const length = parseInt(headRes.headers.get("Content-Length")!, 10)
    return [true, length]
}

export const onRequestPost: PagesFunction<Env> = async (context) => {
    const uuid = context.params.uuid as string

    const req_body = await get_req_body(context.request, PostBody)
    if (!req_body.success) {
        return res.unprocessable_entity("Body verification failed", req_body.issues)
    }
    const { command, copy_source, hash } = req_body.output

    const { success, video, error } = await video_by_uuid_with_hash(context.env.DB, uuid)
    if (!success) {
        return res.db_transaction_error(error)
    }
    if (!video) {
        return res.not_found(`Video ${uuid} not found`)
    }
    if (video.restricted && video.restricted_hash != hash) {
        return res.forbidden("Invalid restricted hash")
    }

    const aws = new AwsClient({
        accessKeyId: context.env.S3_KEY_ID,
        secretAccessKey: context.env.S3_KEY
    });

    const src_url = obj_urls.from_key(context.env, copy_source)
    const dst_url = obj_urls.video(context.env, video);

    if (command == "copy_start") {
        const { part_size } = req_body.output

        const [exists, length] = await get_s3_url_info(aws, src_url)
        try {
            if (!exists) {
                return res.not_found("copy_source does not exist")
            }
        } catch (e) {
            if (e instanceof Error) {
                return res.s3_error(e?.message)
            } else {
                return res.internal_server_error()
            }
        }

        const parts = Math.ceil(length / part_size)

        // init multipart upload
        const init = await aws.fetch(
            `${dst_url}?uploads`,
            { method: 'POST' }
        );
        const init_xml = await init.text();
        const upload_id = /<UploadId>([^<]+)<\/UploadId>/.exec(init_xml)?.[1];

        const urls = [];
        for (let i = 0; i < parts; i++) {
            const url = new URL(dst_url)
            url.searchParams.set("partNumber", (i + 1).toString());
            url.searchParams.set("uploadId", upload_id);

            const from = part_size * i
            const to = Math.min(length, from + part_size) - 1

            const signed = await aws.sign(url.href, {
                method: "PUT",
                headers: {
                    "x-amz-copy-source": copy_source,
                    "x-amz-copy-source-range": `bytes=${from}-${to}`
                },
                aws: { signQuery: true }
            });

            urls.push(signed.url);
        }
        return res.ok({ length, upload_id, urls })
    } else {
        const { etags, upload_id } = req_body.output

        const xml = `<?xml version="1.0" encoding="UTF-8"?>
	<CompleteMultipartUpload>
    ${etags.map((t, i) => (
            `<Part><PartNumber>${i + 1}</PartNumber><ETag>${t}</ETag></Part>`
        )).join("\n")}
    </CompleteMultipartUpload>`;

        const completeRes = await aws.fetch(
            `${dst_url}?uploadId=${encodeURIComponent(upload_id)}`,
            {
                method: 'POST',
                body: xml,
                headers: { 'Content-Type': 'application/xml' }
            }
        );
        const res_xml = await completeRes.text()
        if (completeRes.status != 200) {
            return res.s3_error(
                `Status ${completeRes.status} from S3 while finishing upload`,
                { xml: res_xml }
            )
        }

        const deleteRes = await aws.fetch(src_url, { method: "DELETE" })

        if (!deleteRes.ok && deleteRes.status != 404) {
            const xml = await deleteRes.text()
            return res.s3_error(
                `Status ${deleteRes.status} from S3 while issuing DELETE request`,
                { xml }
            )
        }
        return res.ok()
    }
}

const PutBody = v.object({
    restricted: v.number(),
    hash: v.string(),
})

async function update_restricted(db: D1Database, uuid: string, hash: string | null) {
    const ps = db
        .prepare("UPDATE video SET restricted=?, restricted_hash=? WHERE uuid=UNHEX(?)")
        .bind(hash === null ? 0 : 1, hash, uuid)
    return run_query(ps)
}

export const onRequestPut: PagesFunction<Env> = async (context) => {
    const uuid = context.params.uuid as string

    const req_body = await get_req_body(context.request, PutBody)
    if (!req_body.success) {
        return res.unprocessable_entity("Body verification failed", req_body.issues)
    }
    const { restricted, hash } = req_body.output

    const { success, video, error } = await video_by_uuid_with_hash(context.env.DB, uuid)
    if (!success) {
        return res.db_transaction_error(error)
    }
    if (!video) {
        return res.not_found(`Video ${uuid} not found`)
    }

    if (video.restricted && video.restricted_hash != hash) {
        return res.forbidden("Invalid restricted hash")
    }

    const unrestricted_key = obj_urls.video_key_unrestricted(
        context.env, video.room, video.uuid)
    const restricted_key = obj_urls.video_key_restricted(context.env, video.room, hash)

    let old_key: string
    if (restricted === 0) {
        const ret = await update_restricted(context.env.DB, uuid, null)
        if (!ret.success) {
            return res.db_transaction_error(ret.error)
        }

        old_key = restricted_key
    } else {

        const ret = await update_restricted(context.env.DB, uuid, hash)
        if (!ret.success) {
            return res.db_transaction_error(ret.error)
        }

        old_key = unrestricted_key
    }

    const aws = new AwsClient({
        accessKeyId: context.env.S3_KEY_ID,
        secretAccessKey: context.env.S3_KEY
    });
    const url = obj_urls.from_key(context.env, old_key)
    const [exists] = await get_s3_url_info(aws, url)
    try {
        if (!exists) {
            return res.ok({})
        }
    } catch (e) {
        if (e instanceof Error) {
            return res.s3_error(e?.message)
        } else {
            return res.internal_server_error()
        }
    }

    return res.ok({ copy_source: old_key })
}
