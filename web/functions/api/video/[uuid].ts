interface Env {
    DB: D1Database
}

interface VideoInfo {
    title: string,
    cover: string | null,
    room: number
    timestamp: number
}

async function insert(uuid: string, v: VideoInfo, db: D1Database) {
    const ps = db.prepare(
        "INSERT INTO video (uuid, title, cover, room, timestamp) "
        + "VALUES (UNHEX(?), ?, ?, ?, ?)"
    ).bind(uuid, v.title, v.cover, v.room, v.timestamp)
    try {
        const { success } = await ps.run()
        if (!success) {
            return Response.json({ error: "DB transaction error" }, { status: 500 })
        }
        return Response.json({ success: true })
    } catch (e) {
        return Response.json({ error: "DB transaction error" }, { status: 500 })
    }
}

async function get(id: string, db: D1Database) {
    const ps = db.prepare(
        "SELECT LOWER(HEX(uuid)) as uuid, title, cover, room, timestamp FROM video "
        + "WHERE uuid = UNHEX(?)"
    ).bind(id)

    const { success, results } = await ps.run()
    if (success) {
        if (!results.length) {
            return Response.json({ error: "video not found" }, { status: 404 })
        }
        return Response.json({ success: true, data: results[0] })
    } else {
        return Response.json({ error: "DB transaction error" }, { status: 500 })
    }
}

export const onRequest: PagesFunction<Env> = async (context) => {
    const uuid = context.params.uuid as string

    if (!/^[a-f0-9]{32}$/i.test(uuid)) {
        return Response.json({ error: "invalid uuid format" }, { status: 400 })
    }


    switch (context.request.method) {
        case "POST":
            const video_info = await context.request.json()
            return await insert(uuid, video_info as VideoInfo, context.env.DB)
        case "GET":
            return await get(uuid, context.env.DB)
        default:
            return Response.json({ error: "method not supported" }, { status: 405 })
    }
}
