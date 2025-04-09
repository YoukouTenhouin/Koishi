import { AwsClient } from 'aws4fetch'

interface Env {
    DB: D1Database,
    S3_BUCKET: string,
    S3_ENDPOINT: string,
    S3_KEY_ID: string
    S3_KEY: string,
}

interface VideoInfo {
    title: string,
    cover: string | null,
    room: number
    timestamp: number
}

async function get_video(uuid: string, db: D1Database): Promise<VideoInfo | null> {
    const ps = db.prepare(
        "SELECT title, cover, room, timestamp FROM video WHERE uuid = UNHEX(?)"
    ).bind(uuid)
    const { success, results } = await ps.run()
    if (!success) {
        throw Error("DB transaction failed")
    }
    if (!results.length) {
        return null
    }

    const { title, cover, room, timestamp } = results[0]

    return {
        title: title as string,
        cover: cover as string | null,
        room: room as number,
        timestamp: timestamp as number
    }
}

export const onRequest: PagesFunction<Env> = async (context) => {
    if (context.request.method != "POST") {
        return Response.json({ error: "method not allowed" }, { status: 405 })
    }

    const uuid = context.params.uuid as string

    if (!/^[a-f0-9]{32}$/i.test(uuid)) {
        return Response.json({ error: "invalid uuid format" }, { status: 400 })
    }

    const video = await get_video(uuid, context.env.DB)

    const aws = new AwsClient({
        accessKeyId: context.env.S3_KEY_ID,
        secretAccessKey: context.env.S3_KEY
    });

    const url = `${context.env.S3_ENDPOINT}/${context.env.S3_BUCKET}/${video.room}/${uuid}`
    console.log("URL: ", url)
    const signed = await aws.sign(
        url,
        {
            method: "PUT",
            aws: {
                signQuery: true
            }
        })

    return Response.json({
        success: true,
        data: {
            url: signed.url,
            video,
        }
    })
}
