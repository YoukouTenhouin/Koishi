import { AwsClient } from 'aws4fetch'

import { obj_urls } from '@flib/objects'
import { video_by_uuid } from '@flib/queries'
import { res } from '@flib/responses'
import { Env } from '@flib/types'

export const onRequestPost: PagesFunction<Env> = async (context) => {
    const uuid = context.params.uuid as string

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

    const obj_url = obj_urls.metadata(context.env, video)
    const signed = await aws.sign(obj_url, {
        method: "PUT",
        aws: { signQuery: true }
    });

    return res.ok({ url: signed.url })
}
