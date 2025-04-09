interface Env {
    DB: D1Database
}

interface RoomInfo {
    id: number
    short_id: number
    username: string
    image: string
}

async function insert(room_info: RoomInfo, db: D1Database): Promise<Response> {
    const { id, short_id, username, image } = room_info
    const ps = db.prepare(
        "INSERT INTO room (id, short_id, username, image) VALUES (?, ?, ?, ?)"
    ).bind(id, short_id, username, image)
    try {
        const { success } = await ps.run()
        if (success) {
            return Response.json({ "success": true })
        } else {
            return Response.json({ "error": "DB transaction failed" }, { status: 500 })
        }
    } catch (e) {
        console.error("DB error:", e)
        return Response.json({ "error": `DB transaction failed: ${JSON.stringify(e)}` },
            { status: 500 })
    }
}

async function get(room_id: number, db: D1Database): Promise<Response> {
    const ps = db.prepare(
        "SELECT id, short_id, username, image FROM room WHERE id = ?"
    ).bind(room_id)
    const { success, results } = await ps.run()
    if (success) {
        if (!results.length) {
            return Response.json({ "error": "not found" }, { status: 400 })
        }
        return Response.json({ "success": true, data: results[0] })
    } else {
        return Response.json({ "error": "DB transaction failed" }, { status: 500 })
    }
}

export const onRequest: PagesFunction<Env> = async (context) => {
    switch (context.request.method) {
        case "GET":
            return get(parseInt(context.params.room as string), context.env.DB)
        case "POST":
            const body = await context.request.json<RoomInfo>()
            return insert(body, context.env.DB)
        default:
            return Response.json({ "error": "method not allowed" }, { status: 405 })
    }
}
