import { FC, useEffect, useState } from 'react'
import { Card, Flex, Image, Text } from '@mantine/core'
import { useNavigate, useParams } from 'react-router'

interface VideoListEntry {
    uuid: string,
    title: string,
    cover: string | null
    timestamp: number
}

const VideoEntry: FC<{ video: VideoListEntry }> = ({ video }) => {
    const { uuid, title, cover, timestamp } = video

    const date = new Date(timestamp / 1000)
    const navigate = useNavigate()

    return (
        <Card onClick={() => navigate(`/video/${uuid}`)}>
            <Card.Section>
                <Image
                    width={160}
                    height={90}
                    src={cover ?? "https://placehold.co/704x396?text=NO%20COVER"}
                    referrerPolicy="no-referrer" />
            </Card.Section>

            <Card.Section>
                <Text fw={700}>{title}</Text>
            </Card.Section>

            <Card.Section>
                <Text>{date.toDateString()}</Text>
            </Card.Section>
        </Card >
    )
}

const Room: FC = () => {
    const params = useParams()
    const room_id = params.room!
    const [videos, setVideos] = useState<VideoListEntry[]>([])

    useEffect(() => {
        const loader = async () => {
            const res = await fetch(`/api/room/${room_id}/video`)
            const body = await res.json()
            setVideos(body.data)
        }
        loader()
    }, [room_id])

    return (
        <Flex
            justify="center"
            gap="md"
        >
            {videos.map(v => <VideoEntry key={v.uuid} video={v} />)}
        </Flex>
    )
}

export default Room
