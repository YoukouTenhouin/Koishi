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

export interface Video {
    uuid: string,
    title: string,
    cover: string | null
    room: number
    timestamp: number
}
