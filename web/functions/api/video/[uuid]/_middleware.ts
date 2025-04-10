import { Env } from '@flib/types'
import { res } from '@flib/responses'

export const onRequest: PagesFunction<Env> = async (context) => {
    const uuid = context.params.uuid as string

    if (!/^[a-f0-9]{32}$/i.test(uuid)) {
        return res.bad_request(`Invalid UUID: ${uuid}`)
    }

    return await context.next()
}
