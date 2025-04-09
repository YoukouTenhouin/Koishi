interface Env {
    AUTH_KEY: string
}

export const onRequest: PagesFunction<Env> = async (context) => {
    const auth = context.request.headers.get("authorization")
    if (context.request.method == "GET" || auth == `Bearer ${context.env.AUTH_KEY}`) {
        return await context.next()
    } else {
        return Response.json({
            "error": "unauthorized"
        }, { status: 401 })
    }
}
