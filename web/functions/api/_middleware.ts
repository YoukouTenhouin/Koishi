import { Env } from '@flib/types'
import { res } from '@flib/responses'

export const onRequest: PagesFunction<Env> = async (context) => {
    const auth = context.request.headers.get("authorization")
    if (context.request.method == "GET" || auth == `Bearer ${context.env.AUTH_KEY}`) {
        return await context.next()
    } else {
        return res.unauthorized("Invalid authentication key")
    }
}
