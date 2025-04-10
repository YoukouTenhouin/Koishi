import { run_query } from '@flib/queries'
import { res } from '@flib/responses'
import { Env, Room } from '@flib/types'

export const onRequestGet: PagesFunction<Env> = async (context) => {
    const q_params = new URL(context.request.url).searchParams
    const limit = parseInt(q_params.get("limit") ?? "50", 10)
    const offset = parseInt(q_params.get("offset") ?? "0", 10)

    const ps = context.env.DB.prepare(
        "SELECT id, short_id, username, image FROM room LIMIT ? OFFSET ?"
    ).bind(limit, offset)

    const ret = await run_query<Room>(ps);
    if (!ret.success) {
        return res.db_transaction_error(ret.error)
    }

    return res.ok(ret.results)
}
