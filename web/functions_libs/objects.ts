import { Env, Video } from '@flib/types'

export namespace obj_urls {
    export function cover(env: Env, hash: string) {
        return `${env.S3_ENDPOINT}/${env.S3_BUCKET}/cover/${hash.toLowerCase()}`
    }

    export function video(env: Env, video: Video) {
        return `${env.S3_ENDPOINT}/${env.S3_BUCKET}/video/${video.room}/${video.uuid.toLowerCase()}`
    }

    export function metadata(env: Env, video: Video) {
        return `${env.S3_ENDPOINT}/${env.S3_BUCKET}/metadata/${video.uuid.toLowerCase()}`
    }
}
