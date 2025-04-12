import { Env, Video } from '@flib/types'

export namespace obj_urls {
    export function from_key(env: Env, key: string) {
        return new URL(key, `${env.S3_ENDPOINT}`).href
    }

    export function cover_key(env: Env, hash: string) {
        return `/${env.S3_BUCKET}/cover/${hash.toLowerCase()}`
    }

    export function video_key_unrestricted(env: Env, room: string | number, uuid: string) {
        return `/${env.S3_BUCKET}/video/${room}/${uuid.toLowerCase()}`
    }

    export function video_key_restricted(env: Env, room: string | number, hash: string) {
        return `/${env.S3_BUCKET}/video_restricted/${room}/${hash.toLowerCase()}`
    }

    export function video_key(env: Env, video: Video) {
        if (video.restricted) {
            return video_key_restricted(env, video.room, video.restricted_hash)
        } else {
            return video_key_unrestricted(env, video.room, video.uuid)
        }
    }

    export function metadata_key(env: Env, video: Video) {
        return `/${env.S3_BUCKET}/metadata/${video.uuid.toLowerCase()}`
    }

    export function cover(env: Env, hash: string) {
        return from_key(env, cover_key(env, hash))
    }

    export function video(env: Env, video: Video) {
        return from_key(env, video_key(env, video))
    }

    export function metadata(env: Env, video: Video) {
        return from_key(env, metadata_key(env, video))
    }
}
