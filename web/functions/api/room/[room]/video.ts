import { run_query } from '@flib/queries'
import { res } from '@flib/responses'
import { Env, Video } from '@flib/types'

interface ResultEntry extends Omit<Video, "room" | "restricted_hash"> {
    parts: number
}

export const onRequestGet: PagesFunction<Env> = async (context) => {
    const room = parseInt(context.params.room as string)

    const q_params = new URL(context.request.url).searchParams
    const limit = Math.min(100, parseInt(q_params.get("limit") ?? "50", 10))
    const offset = parseInt(q_params.get("offset") ?? "0", 10)

    const ps = context.env.DB.prepare(`
        SELECT LOWER(HEX(uuid)) as uuid, title, cover, stream_time, record_time, restricted, parts
	FROM (
	   SELECT *,
	     ROW_NUMBER() OVER (PARTITION BY stream_time ORDER BY record_time ASC) AS row_num,
	     COUNT(*) OVER (PARTITION BY stream_time) as parts
	   FROM video
        ) WHERE room = ? AND row_num = 1 ORDER BY stream_time DESC LIMIT ? OFFSET ?
    `).bind(room, limit, offset)
    const ret = await run_query<ResultEntry>(ps)
    if (!ret.success) {
        return res.db_transaction_error(ret.error)
    }
    return res.ok(ret.results)
}
