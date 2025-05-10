export interface Env {
    DB: D1Database
    AUTH_KEY: string
    S3_BUCKET: string
    S3_ENDPOINT: string
    S3_KEY_ID: string
    S3_KEY: string
}

export interface Room {
    id: number
    short_id: number | null
    username: string
    image: string
}

interface VideoCommon {
    uuid: string,
    title: string,
    cover: string | null
    room: number
    stream_time: number
    record_time: number
}

interface UnrestrictedVideo extends VideoCommon {
    restricted: 0
    restricted_hash: null
}

interface RestrictedVideo extends VideoCommon {
    restricted: 1
    restricted_hash: string
}

export type Video = UnrestrictedVideo | RestrictedVideo
