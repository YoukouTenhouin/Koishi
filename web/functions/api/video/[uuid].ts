import * as v from 'valibot'

import { Env } from '@flib/types'
import { run_query, video_by_uuid } from '@flib/queries'
import { get_req_body } from '@flib/requests'
import { res } from '@flib/responses'

const ReqInsertCommon = v.object({
    title: v.pipe(v.string(), v.nonEmpty()),
    cover: v.nullable(v.string()),
    room: v.number(),
    stream_time: v.number(),
    record_time: v.number(),
})

const ReqInsertUnrestricted = v.object({
    ...ReqInsertCommon.entries,
    restricted: v.optional(v.literal(0)),
    restricted_hash: v.optional(v.null()),
})
const ReqInsertRestricted = v.object({
    ...ReqInsertCommon.entries,
    restricted: v.literal(1),
    restricted_hash: v.string(),
})
const ReqInsert = v.variant('restricted', [
    ReqInsertUnrestricted,
    ReqInsertRestricted
])

const ReqUpdate = v.partial(v.object({
    title: v.string(),
    cover: v.string(),
    stream_time: v.number(),
    record_time: v.number(),
}))

async function insert(uuid: string, v: v.InferOutput<typeof ReqInsert>, db: D1Database) {
    const ps = db.prepare(
        "INSERT OR IGNORE INTO video "
        + "(uuid, title, cover, room, stream_time, record_time, restricted, restricted_hash) "
        + "VALUES (UNHEX(?), ?, ?, ?, ?, ?, ?, ?)"
    ).bind(uuid, v.title, v.cover, v.room, v.stream_time, v.record_time,
        v.restricted ?? 0, v.restricted_hash ?? null)
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

async function update(id: string, d: v.InferOutput<typeof ReqUpdate>, db: D1Database) {
    const { success: fetch_success, video, error: fetch_error } = await video_by_uuid(db, id)
    if (fetch_success) {
        if (video === null) {
            return res.not_found(`Video ${id} not found`)
        }
    } else {
        return res.db_transaction_error(fetch_error)
    }

    const { title, cover, stream_time, record_time } = d

    const ps = db.prepare(
        "UPDATE video SET title=?, cover=?, stream_time=?, record_time=? WHERE uuid=UNHEX(?)"
    ).bind(
        title ?? video.title,
        cover ?? video.cover,
        stream_time ?? video.stream_time,
        record_time ?? video.record_time,
        id
    )
    const ret = await run_query(ps)
    if (!ret.success) {
        return res.db_transaction_error(ret.error)
    }
    return res.ok()
}

async function post(uuid: string, db: D1Database, request: Body) {
    const { success, output } = await get_req_body(request, ReqInsert)
    if (!success) {
        return res.unprocessable_entity()
    }
    return await insert(uuid, output, db)
}

async function get(uuid: string, db: D1Database) {
    return await fetch(uuid, db)
}

async function put(uuid: string, db: D1Database, request: Body) {
    const { success, output } = await get_req_body(request, ReqUpdate)
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
