import * as v from 'valibot'

export namespace schemas {
    export const Room = v.object({
        id: v.number(),
        short_id: v.nullable(v.number()),
        username: v.string(),
        image: v.string()
    })

    export const Video = v.object({
        uuid: v.string(),
        title: v.string(),
        cover: v.nullable(v.string()),
        room: v.number(),
        timestamp: v.number(),
        restricted: v.number(),
    })
}

export namespace SchemaTypes {
    export type Room = v.InferOutput<typeof schemas.Room>
    export type Video = v.InferOutput<typeof schemas.Video>
}
