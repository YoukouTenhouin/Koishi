interface Env {
    DB: D1Database
}

export const onRequest: PagesFunction<Env> = async (context) => {
    if (context.request.method != "GET") {
        return Response.json({ "error": "method not allowed" }, { status: 405 })
    }

    const ps = context.env.DB.prepare("SELECT id, short_id, username, image FROM room")
    const { success, results } = await ps.run();

    if (!success) {
        return Response.json({ "error": "DB transaction error" }, { status: 500 })
    }

    return Response.json({ success: true, data: results })
}
