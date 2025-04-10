import * as v from 'valibot'

export async function get_req_body<T extends v.BaseSchema<
    unknown, unknown, v.BaseIssue<unknown>
>>(request: Body, schema: T) {
    const json = await request.json()
    return v.safeParse(schema, json)
}
