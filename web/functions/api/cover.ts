import { AwsClient } from 'aws4fetch'

interface Env {
    DB: D1Database,
    S3_BUCKET: string,
    S3_ENDPOINT: string,
    S3_KEY_ID: string
    S3_KEY: string,
}

interface RequestBody {
    hash: string
}

export const onRequest: PagesFunction<Env> = async (context) => {
    if (context.request.method != "POST") {
        return Response.json({ error: "method not allowed" }, { status: 405 })
    }

    const { hash } = await context.request.json<RequestBody>()

    const aws = new AwsClient({
        accessKeyId: context.env.S3_KEY_ID,
        secretAccessKey: context.env.S3_KEY
    });

    const obj_url = `${context.env.S3_ENDPOINT}/${context.env.S3_BUCKET}/cover/${hash.toLowerCase()}`
    const headRes = await aws.fetch(obj_url, { method: 'HEAD' });

    if (headRes.status === 200) {
        return Response.json({ success: true, data: { exists: true, url: null } });
    }

    if (headRes.status !== 404) {
        return Response.json({ error: "unexpected response from S3" }, { status: 500 });
    }

    const signed = await aws.sign(obj_url, {
        method: "PUT",
        aws: { signQuery: true }
    });

    return Response.json({
        success: true,
        data: {
            exists: false,
            url: signed.url
        }
    })
}
