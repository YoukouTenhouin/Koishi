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

export function getVideoURL(room_id: number | string, uuid: string) {
    return getObjectURL(`/video/${room_id}/${uuid}`)
}
