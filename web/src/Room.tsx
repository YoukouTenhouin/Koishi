import { FC, useEffect, useState } from 'react'
import { useDisclosure } from '@mantine/hooks'
import { AppShell, Burger, Group, Stack, Card, Flex, Image, Text, Skeleton } from '@mantine/core'
import { useNavigate, useParams } from 'react-router'

import SiteTitle from './components/SiteTitle'

interface VideoListEntry {
    uuid: string,
    title: string,
    cover: string | null
    timestamp: number
}

interface RoomInfo {
    id: number
    short_id: number | null
    username: string
    image: string
}

function phImg(width: number, height: number, text: string) {
    return `https://placehold.co/${width}x${height}?text=${encodeURIComponent(text)}`
}

const cdn_base_url = import.meta.env.VITE_KOISHI_CDN_PREFIX;

function coverUrl(cover: string | null) {
    return cover && new URL(`/cover/${cover}`, cdn_base_url).href
}

const VideoEntry: FC<{ video: VideoListEntry }> = ({ video }) => {
    const { uuid, title, cover, timestamp } = video

    const date = new Date(timestamp)
    const navigate = useNavigate()

    return (
        <Card
            shadow="sm"
            padding="lg"
            withBorder
            onClick={() => navigate(`/video/${uuid}`)}>
            <Card.Section>
                <Image
                    width={320}
                    height={180}
                    src={coverUrl(cover) ?? phImg(320, 180, "NO COVER")}
                    referrerPolicy="no-referrer" />
            </Card.Section>

            <Card.Section>
                <Text fw={700}>{title}</Text>
            </Card.Section>

            <Card.Section>
                <Text>直播时间: {date.toDateString()}</Text>
            </Card.Section>
        </Card >
    )
}

const Room: FC = () => {
    const params = useParams()
    const room_id = params.room!
    const [roomInfo, setRoomInfo] = useState<RoomInfo | null>(null)
    const [videos, setVideos] = useState<VideoListEntry[]>([])
    const [asideOpened, { toggle }] = useDisclosure()

    const loading = !!roomInfo

    useEffect(() => {
        const loader = async () => {
            const room_info_res = await fetch(`/api/room/${room_id}`)
            const room_info_body = await room_info_res.json()
            setRoomInfo(room_info_body.result)

            const video_res = await fetch(`/api/room/${room_id}/video`)
            const video_body = await video_res.json()
            setVideos(video_body.result)
        }
        loader()
    }, [room_id])

    return (
        <AppShell
            header={{ height: 60 }}
            aside={{
                width: 200,
                breakpoint: 'sm',
                collapsed: { mobile: !asideOpened }
            }}
            padding="md">
            <AppShell.Header>
                <Group h="100%" px="md" justify="space-between">
                    <SiteTitle />
                    <Burger
                        opened={asideOpened}
                        onClick={toggle}
                        hiddenFrom="sm"
                        size="sm"
                    />
                </Group>
            </AppShell.Header>
            <AppShell.Aside>
                <Stack
                    align="center"
                    p="lg">
                    <Skeleton
                        w="auto"
                        visible={!loading}>
                        <Image
                            w={192}
                            h={192}
                            src={roomInfo?.image ?? phImg(192, 192, "No Image")}
                            referrerPolicy="no-referrer"
                        />
                    </Skeleton>
                    <Skeleton visible={!loading}>
                        <Text ta="center" fw={700}>{roomInfo?.username ?? "LOADING"}</Text>
                    </Skeleton>
                    <Skeleton visible={!loading}>
                        <Text size="sm" c="dimmed" ta="center">
                            {roomInfo?.short_id ?? roomInfo?.id ?? "114514"}
                        </Text>
                    </Skeleton>
                </Stack>
            </AppShell.Aside>
            <AppShell.Main>
                <Flex
                    gap="md"
                    wrap="wrap"
                >
                    {videos.map(v => <VideoEntry key={v.uuid} video={v} />)}
                </Flex>
            </AppShell.Main>
        </AppShell>
    )
}

export default Room
