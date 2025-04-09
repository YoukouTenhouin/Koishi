interface Env {
    DB: D1Database
}

interface VideoInfo {
    uuid: string,
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

async function fetch_video(id: string, db: D1Database): Promise<[boolean, VideoInfo | null]> {
    const ps = db.prepare(
        "SELECT LOWER(HEX(uuid)) as uuid, title, cover, room, timestamp FROM video "
        + "WHERE uuid = UNHEX(?)"
    ).bind(id)

    const { success, results } = await ps.run()
    if (!success) {
        return [false, null]
    } else if (!results.length) {
        return [true, null]
    } else {
        const { uuid, title, cover, room, timestamp } = results[0]
        return [true, {
            uuid: uuid as string,
            title: title as string,
            cover: cover as string | null,
            room: room as number,
            timestamp: timestamp as number,
        }]
    }
}

async function get(id: string, db: D1Database) {
    const [success, video] = await fetch_video(id, db)
    if (success) {
        if (video === null) {
            return Response.json({ error: "video not found" }, { status: 404 })
        }
        return Response.json({ success: true, data: video })
    } else {
        return Response.json({ error: "DB transaction error" }, { status: 500 })
    }
}

async function update(id: string, d: Partial<VideoInfo>, db: D1Database) {
    const [fetch_success, video] = await fetch_video(id, db)
    if (fetch_success) {
        if (video === null) {
            return Response.json({ error: "video not found" }, { status: 404 })
        }
    } else {
        return Response.json({ error: "DB transaction error" }, { status: 500 })
    }

    const { title, cover, timestamp } = d

    const ps = db.prepare(
        "UPDATE video SET title=?, cover=?, timestamp=? WHERE uuid=UNHEX(?)"
    ).bind(
        title ?? video.title,
        cover ?? video.cover,
        timestamp ?? video.timestamp,
        id
    )
    const { success } = await ps.run()
    if (!success) {
        return Response.json({ error: "DB transaction error" }, { status: 500 })
    }
    return Response.json({ success: true })
}

export const onRequest: PagesFunction<Env> = async (context) => {
    const uuid = context.params.uuid as string

    if (!/^[a-f0-9]{32}$/i.test(uuid)) {
        return Response.json({ error: "invalid uuid format" }, { status: 400 })
    }

    switch (context.request.method) {
        case "POST":
            const video_info = await context.request.json<VideoInfo>()
            return await insert(uuid, video_info, context.env.DB)
        case "GET":
            return await get(uuid, context.env.DB)
        case "PUT":
            const diff = await context.request.json<Partial<VideoInfo>>()
            return await update(uuid, diff, context.env.DB)
        default:
            return Response.json({ error: "method not supported" }, { status: 405 })
    }
}
