import { restricted_hash } from "./cryptography";
import { SchemaTypes } from "./schemas";

const CDN_BASE_URL = import.meta.env.VITE_KOISHI_CDN_PREFIX;

export function getObjectURL(path: string) {
    return new URL(path, CDN_BASE_URL).href
}

export function getCoverURL(hash: string) {
    return getObjectURL(`/cover/${hash}`)
}

export function getMetadataURL(uuid: string) {
    return getObjectURL(`/metadata/${uuid}`)
}

export function getUnrestrictedVideoURL(video: SchemaTypes.Video) {
    return getObjectURL(`/video/${video.room}/${video.uuid}`)
}

export function getRestrictedVideoURL(video: SchemaTypes.Video, hash: string) {
    return getObjectURL(`/video_restricted/${video.room}/${hash}`)
}
