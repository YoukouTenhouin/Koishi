interface Env {
    DB: D1Database
}

export const onRequest: PagesFunction<Env> = async (context) => {
    if (context.request.method != "GET") {
        return Response.json({ "error": "method not allowed" }, { status: 405 })
    }

    const room = parseInt(context.params.room as string)

    const ps = context.env.DB.prepare(
        "SELECT LOWER(HEX(uuid)) as uuid, title, cover, timestamp FROM video "
        + "WHERE room = ? ORDER BY timestamp DESC"
    ).bind(room)
    const { success, results } = await ps.run()
    if (!success) {
        return Response.json({ "error": "DB transaction error" }, { status: 500 })
    }

    return Response.json({ success: true, data: results })
}
