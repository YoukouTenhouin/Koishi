import { AwsClient } from 'aws4fetch'
import * as v from 'valibot'

import { obj_urls } from '@flib/objects'
import { video_by_uuid } from '@flib/queries'
import { get_req_body } from '@flib/requests'
import { res } from '@flib/responses'
import { Env } from '@flib/types'

const ReqBody = v.object({
    size: v.number(),
    part_size: v.number()
})

export const onRequestPost: PagesFunction<Env> = async (context) => {
    const uuid = context.params.uuid as string

    const req_body = await get_req_body(context.request, ReqBody)
    if (!req_body.success) {
        return res.unprocessable_entity()
    }
    const { size, part_size } = req_body.output
    const parts = Math.ceil(size / part_size)

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

    const obj_url = obj_urls.video(context.env, video)

    // init multipart upload
    const init = await aws.fetch(
        `${obj_url}?uploads`,
        { method: 'POST' }
    );
    const init_xml = await init.text();
    const upload_id = /<UploadId>([^<]+)<\/UploadId>/.exec(init_xml)?.[1];

    const urls = [];
    for (let i = 1; i <= parts; i++) {
        const url = new URL(obj_url)
        url.searchParams.set("partNumber", i.toString());
        url.searchParams.set("uploadId", upload_id);

        const signed = await aws.sign(url.href, {
            method: "PUT",
            aws: { signQuery: true }
        });

        urls.push(signed.url);
    }

    return res.ok({ urls, upload_id, video })
}
