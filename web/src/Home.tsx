import { FC, useEffect, useState } from 'react'
import { AppShell, Card, Container, Flex, Group, Image, Loader, Text } from '@mantine/core'
import { useNavigate } from 'react-router'
import * as v from 'valibot'

import SiteTitle from '@components/SiteTitle'
import { APIError, FCWithAPI, useAPI } from '@lib/api'
import { schemas } from '@lib/schemas'
import ErrorPage from '@components/Error'

interface RoomListEntry {
    id: number,
    short_id: number | null,
    username: string,
    image: string
}

const Entry: FC<{ room: RoomListEntry }> = ({ room }) => {
    const { id, short_id, username, image } = room

    const navigate = useNavigate()

    return (
        <Card shadow="sm" padding="lg" withBorder onClick={() => navigate(`/room/${id}`)}>
            <Card.Section>
                <Group>
                    <Image height={128} src={image} referrerPolicy="no-referrer" />
                </Group>
            </Card.Section>

            <Card.Section p="md">
                <Text fw={700}>{username}</Text>
                <Text>{`${id}${short_id ? ", " + short_id : ""}`}</Text>
            </Card.Section>
        </Card >
    )
}

const RoomList = v.array(schemas.Room)

const HomeView: FCWithAPI<typeof RoomList> = ({ loading, result, error }) => {
    if (loading) {
        return <Loader />
    } else if (error) {
        return <ErrorPage error={error} />
    }

    return (
        <Flex
            gap="md"
            wrap="wrap"
        >
            {result!.map(r => <Entry key={r.id} room={r} />)}
        </Flex>
    )
}

const Home: FC = () => {
    const api_result = useAPI(RoomList, "/api/room")

    return (
        <AppShell
            header={{ height: 60 }}
            padding="md">
            <AppShell.Header>
                <Group h="100%" px="md">
                    <SiteTitle />
                </Group>
            </AppShell.Header>
            <AppShell.Main>
                <HomeView {...api_result} />
            </AppShell.Main>
        </AppShell>
    )
}

export default Home
