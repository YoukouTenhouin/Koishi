import { AwsClient } from 'aws4fetch'
import * as v from 'valibot'

import { Env } from '@flib/types'
import { get_req_body } from '@flib/requests'
import { res } from '@flib/responses'
import { obj_urls } from '@flib/objects'

const ReqBody = v.object({
    hash: v.pipe(v.string(), v.nonEmpty())
})

export const onRequestPost: PagesFunction<Env> = async (context) => {
    const { success, output } = await get_req_body(context.request, ReqBody)
    if (!success) {
        return res.unprocessable_entity()
    }
    const { hash } = output

    const aws = new AwsClient({
        accessKeyId: context.env.S3_KEY_ID,
        secretAccessKey: context.env.S3_KEY
    });

    const obj_url = obj_urls.cover(context.env, hash)
    const headRes = await aws.fetch(obj_url, { method: 'HEAD' });

    if (headRes.status === 200) {
        return res.ok({ exists: true, url: null });
    }

    if (headRes.status !== 404) {
        const xml = await headRes.text()
        return res.s3_error("Unexpected response from S3", { xml })
    }

    const signed = await aws.sign(obj_url, {
        method: "PUT",
        aws: { signQuery: true }
    });

    return res.ok({
        exists: false,
        url: signed.url
    })
}
