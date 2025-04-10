import { Video } from '@flib/types'

// Run a query, and catch DB errors as return value
export async function run_query<T>(stmt: D1PreparedStatement): Promise<
    D1Result<T> | { success: false, error?: string }
> {
    try {
        return await stmt.run<T>()
    } catch (e) {
        if (!e.message.includes("D1_ERROR")) {
            throw e
        }
        return {
            success: false,
            error: e.message
        }
    }
}

export async function video_by_uuid(db: D1Database, uuid: string): Promise<{
    success: boolean
    video: Video | null
    error?: string
}> {
    const ps = db.prepare(
        "SELECT LOWER(HEX(uuid)) as uuid, title, cover, room, timestamp FROM video "
        + "WHERE uuid = UNHEX(?)"
    ).bind(uuid)

    const ret = await run_query<Video>(ps)
    if (!ret.success) {
        return { success: false, video: null, error: ret.error }
    } else if (!ret.results.length) {
        return { success: true, video: null }
    } else {
        return { success: true, video: ret.results[0] }
    }
}
