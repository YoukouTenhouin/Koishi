import { FC } from "react"
import { Container, Text } from "@mantine/core"

import { APIError } from "@lib/api"

const ErrorPage: FC<{ error: APIError }> = ({ error }) => {
    return (
        <Container>
            <h1>{error.status}: {error.type.toUpperCase()}</h1>
            {error.reason && <Text>{error.reason}</Text>}
        </Container>
    )
}

export default ErrorPage
