import * as v from 'valibot'

import { run_query } from '@flib/queries'
import { get_req_body } from '@flib/requests'
import { res } from '@flib/responses'
import { Env, Room } from '@flib/types'

const ReqBody = v.object({
    id: v.number(),
    short_id: v.nullable(v.number()),
    username: v.string(),
    image: v.string()
})

async function insert(room: Room, db: D1Database): Promise<Response> {
    const { id, short_id, username, image } = room
    const ps = db.prepare(
        "INSERT OR IGNORE INTO room (id, short_id, username, image) VALUES (?, ?, ?, ?)"
    ).bind(id, short_id, username, image)
    const ret = await run_query(ps)
    if (!ret.success) {
        return res.db_transaction_error(ret.error)
    } else if (ret.meta.changes == 0) {
        return res.conflict(`Room ${room.id} already exists`)
    }
    return res.ok()
}

async function get(room_id: number, db: D1Database): Promise<Response> {
    const ps = db.prepare(
        "SELECT id, short_id, username, image FROM room WHERE id = ?"
    ).bind(room_id)
    const ret = await run_query<Room>(ps)
    if (!ret.success) {
        return res.db_transaction_error(ret.error)
    } else if (!ret.results.length) {
        return res.not_found(`Room ${room_id} not found`)
    }
    return res.ok(ret.results[0])
}

export const onRequest: PagesFunction<Env> = async (context) => {
    switch (context.request.method) {
        case "GET":
            return get(parseInt(context.params.room as string), context.env.DB)
        case "POST":
            const { success, output: body } = await get_req_body(context.request, ReqBody)
            if (!success) {
                return res.unprocessable_entity()
            }
            return insert(body, context.env.DB)
        default:
            return res.method_not_allowed()
    }
}
