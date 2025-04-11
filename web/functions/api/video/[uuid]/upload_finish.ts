import { AwsClient } from 'aws4fetch'
import * as v from 'valibot'

import { obj_urls } from '@flib/objects'
import { video_by_uuid } from '@flib/queries'
import { get_req_body } from '@flib/requests'
import { res } from '@flib/responses'
import { Env } from '@flib/types'

const ReqBody = v.object({
    upload_id: v.string(),
    etags: v.array(v.string())
})

export const onRequestPost: PagesFunction<Env> = async (context) => {
    const uuid = context.params.uuid as string

    const { success: parse_success, output } = await get_req_body(context.request, ReqBody)
    if (!parse_success) {
        return res.unprocessable_entity()
    }

    const { upload_id, etags } = output

    const { success, video, error } = await video_by_uuid(context.env.DB, uuid)
    if (!success) {
        return res.db_transaction_error(error)
    }
    if (!video) {
        return res.not_found(`Video ${uuid} not found`)
    }

    const aws = new AwsClient({
        accessKeyId: context.env.S3_KEY_ID,
        secretAccessKey: context.env.S3_KEY
    });

    const xml = `<?xml version="1.0" encoding="UTF-8"?>
    <CompleteMultipartUpload>
    ${etags.map((t, i) => (
        `<Part><PartNumber>${i + 1}</PartNumber><ETag>${t}</ETag></Part>`
    )).join("\n")}
    </CompleteMultipartUpload>`;

    const obj_url = obj_urls.video(context.env, video)

    const complete = await aws.fetch(
        `${obj_url}?uploadId=${encodeURIComponent(upload_id)}`,
        {
            method: 'POST',
            body: xml,
            headers: { 'Content-Type': 'application/xml' }
        }
    );

    const res_text = await complete.text();
    if (complete.status != 200) {
        return res.s3_error(`Status ${complete.status} from S3`, { xml: res_text })
    }
    return res.ok()
}
