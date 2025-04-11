import { useEffect, useState, FC } from 'react';
import * as v from 'valibot';

const APIErrorResult = v.object({
    error: v.string(),
    message: v.undefinedable(v.string()),
    details: v.undefinedable(v.any())
})

function APIResponse<
    TSchema extends v.BaseSchema<unknown, unknown, v.BaseIssue<unknown>>
>(value_schema: TSchema) {
    return v.object({
        result: value_schema
    })
}

function APIResult<
    TSchema extends v.BaseSchema<unknown, unknown, v.BaseIssue<unknown>>
>(value_schema: TSchema) {
    return v.union([APIResponse(value_schema), APIErrorResult])
}

export class APIError extends Error {
    constructor(
        public status: number,
        public type: string,
        public reason?: string,
        public details?: any
    ) {
        super(`APIError: ${type}`)
        this.name = "APIError"
    }
}

export async function parseResponse<
    TSchema extends v.BaseSchema<unknown, unknown, v.BaseIssue<unknown>>
>(schema: TSchema, res: Response) {
    const res_body = await res.json()

    const result = APIResult(schema)
    const output = v.parse(result, res_body)

    if ("error" in output) {
        throw new APIError(res.status, output.error, output.message, output.details)
    }

    return output.result
}

export async function requestAPI<
    TSchema extends v.BaseSchema<unknown, unknown, v.BaseIssue<unknown>>
>(schema: TSchema, input: RequestInfo | URL, init?: RequestInit) {
    const res = await fetch(input, init)
    return await parseResponse(schema, res)
}

export function useAPI<
    TSchema extends v.BaseSchema<unknown, unknown, v.BaseIssue<unknown>>
>(schema: TSchema, input: RequestInfo | URL, init?: RequestInit) {
    const [loading, setLoading] = useState(true)
    const [result, setResult] = useState<v.InferOutput<TSchema> | null>(null)
    const [error, setError] = useState<APIError | null>(null)

    useEffect(() => {
        const loader = async () => {
            setLoading(true)
            setResult(null)
            setError(null)

            try {
                const result = await requestAPI(schema, input, init)
                setLoading(false)
                setResult(result)
            } catch (e) {
                if (e instanceof APIError) {
                    setLoading(false)
                    setError(e)
                } else {
                    throw e
                }
            }
        }
        loader()
    }, [schema, input, init])

    return { loading, result, error }
}

export type FCWithAPI<
    TSchema extends v.BaseSchema<unknown, unknown, v.BaseIssue<unknown>>,
    P = {}
> = FC<ReturnType<typeof useAPI<TSchema>> & P>
