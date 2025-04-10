import * as v from 'valibot'

import { Env, Video } from '@flib/types'
import { run_query, video_by_uuid } from '@flib/queries'
import { get_req_body } from '@flib/requests'
import { res } from '@flib/responses'

async function insert(uuid: string, v: Omit<Video, "uuid">, db: D1Database) {
    const ps = db.prepare(
        "INSERT OR IGNORE INTO video (uuid, title, cover, room, timestamp) "
        + "VALUES (UNHEX(?), ?, ?, ?, ?)"
    ).bind(uuid, v.title, v.cover, v.room, v.timestamp)
    const ret = await run_query(ps)
    if (!ret.success) {
        return res.db_transaction_error(ret.error)
    } else if (ret.meta.changes == 0) {
        return res.conflict(`Video ${uuid} already exists`)
    }
    return res.ok()
}

async function fetch(id: string, db: D1Database) {
    const { success, video, error } = await video_by_uuid(db, id)
    if (success) {
        if (video === null) {
            return res.not_found(`Video ${id} not found`)
        }
        return res.ok(video)
    } else {
        return res.db_transaction_error(error)
    }
}

async function update(id: string, d: Partial<Video>, db: D1Database) {
    const { success: fetch_success, video, error: fetch_error } = await video_by_uuid(db, id)
    if (fetch_success) {
        if (video === null) {
            return res.not_found(`Video ${id} not found`)
        }
    } else {
        return res.db_transaction_error(fetch_error)
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
    const ret = await run_query(ps)
    if (!ret.success) {
        return res.db_transaction_error(ret.error)
    }
    return res.ok()
}

const ReqBody = v.object({
    title: v.pipe(v.string(), v.nonEmpty()),
    cover: v.nullable(v.string()),
    room: v.number(),
    timestamp: v.number()
})

async function post(uuid: string, db: D1Database, request: Body) {
    const { success, output } = await get_req_body(request, ReqBody)
    if (!success) {
        return res.unprocessable_entity()
    }
    return await insert(uuid, output, db)
}

async function get(uuid: string, db: D1Database) {
    return await fetch(uuid, db)
}

async function put(uuid: string, db: D1Database, request: Body) {
    const { success, output } = await get_req_body(request, v.partial(ReqBody))
    if (!success) {
        return res.unprocessable_entity()
    }
    return await update(uuid, output, db)
}

export const onRequest: PagesFunction<Env> = async (context) => {
    const uuid = context.params.uuid as string
    if (!/^[a-f0-9]{32}$/i.test(uuid)) {
        return Response.json({ error: "invalid uuid format" }, { status: 400 })
    }

    switch (context.request.method) {
        case "POST":
            return await post(uuid, context.env.DB, context.request)
        case "GET":
            return await get(uuid, context.env.DB)
        case "PUT":
            return await put(uuid, context.env.DB, context.request)
        default:
            return res.method_not_allowed()
    }
}
