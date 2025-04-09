import { FC, useEffect, useState } from 'react'
import { Card, Flex, Group, Image, Text } from '@mantine/core'
import { useNavigate } from 'react-router'

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

const Home: FC = () => {
    const [rooms, setRooms] = useState<RoomListEntry[]>([])

    useEffect(() => {
        const loader = async () => {
            const res = await fetch("/api/room")
            const body = await res.json()

            setRooms(body.data as RoomListEntry[])
        }
        loader()
    }, [])

    return (
        <Flex
            gap="md"
            wrap="wrap"
            justify="center"
        >
            {rooms.map(r => <Entry key={r.id} room={r} />)}
        </Flex>
    )
}

export default Home
