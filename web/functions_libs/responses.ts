function error_wrapper(status: number, err_type: string) {
    return (message?: string, details?: any) => Response.json({
        error: err_type, message, details
    }, { status })
}

export namespace res {
    export function ok(result: any | null = null) {
        return Response.json({ result })
    }

    export const bad_request = error_wrapper(400, "bad_request")
    export const unauthorized = error_wrapper(401, "unauthorized")
    export const not_found = error_wrapper(404, "not_found")
    export const method_not_allowed = error_wrapper(405, "method_not_allowed")
    export const conflict = error_wrapper(409, "conflict")
    export const unprocessable_entity = error_wrapper(422, "unprocessable_entity")
    export const internal_server_error = error_wrapper(500, "internal_server_error")

    export const db_transaction_error = error_wrapper(500, "db_transaction_error")
    export const s3_error = error_wrapper(500, "s3_error")
}
