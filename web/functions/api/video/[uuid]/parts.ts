import { Env, Video } from '@flib/types'

import { video_by_uuid_with_hash, run_query } from '@flib/queries'
import { res } from '@flib/responses'

export const onRequestGet: PagesFunction<Env> = async (context) => {
    const uuid = context.params.uuid as string

    const { success, video, error } = await video_by_uuid_with_hash(context.env.DB, uuid)
    if (!success) {
        return res.db_transaction_error(error)
    }
    if (!video) {
        return res.not_found(`Video ${uuid} not found`)
    }

    const ps = context.env.DB.prepare(`
    SELECT LOWER(HEX(uuid)) as uuid, title, cover, stream_time, record_time, restricted FROM video
    WHERE room = ? AND stream_time = ? ORDER BY record_time ASC
    `).bind(video.room, video.stream_time)

    const ret = await run_query<Omit<Video, "room" | "restricted_hash">>(ps)
    if (!ret.success) {
        return res.db_transaction_error(ret.error)
    }
    return res.ok(ret.results)
}
