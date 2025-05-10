import { FC, useCallback, useEffect, useState } from 'react'
import { useDisclosure, useInViewport } from '@mantine/hooks'
import {
    AppShell,
    Burger,
    Card,
    Flex,
    Group,
    Image,
    Loader,
    Skeleton,
    Stack,
    Text,
} from '@mantine/core'
import { useNavigate, useParams } from 'react-router'
import * as v from 'valibot'

import SiteTitle from '@components/SiteTitle'
import Cover from '@components/Cover'
import { APIError, requestAPI, useAPI } from '@lib/api'
import { schemas } from '@lib/schemas'
import ErrorPage from '@components/Error'
import { date_stamp } from '@lib/chrono'

const VideoListEntry = v.omit(schemas.Video, ["room"])
type VideoListEntry = v.InferOutput<typeof VideoListEntry>
const VideoList = v.array(VideoListEntry)
type VideoList = v.InferOutput<typeof VideoList>

function phImg(width: number, height: number, text: string) {
    return `https://placehold.co/${width}x${height}?text=${encodeURIComponent(text)}`
}

const VideoLoader: FC<{
    list: VideoList
    onLoad(): void
}> = ({ list, onLoad }) => {
    const { ref, inViewport } = useInViewport()

    useEffect(() => {
        if (!inViewport) return

        onLoad()
    }, [list, inViewport])

    return (
        <Group ref={ref}>
            <Loader />
            <Text>少女折寿中...</Text>
        </Group>
    )
}

const VideoEntry: FC<{
    video: VideoListEntry
}> = ({ video }) => {
    const { uuid, title, stream_time } = video

    const stream_date = new Date(stream_time)
    const navigate = useNavigate()

    return (
        <Card
            shadow="sm"
            padding="lg"
            withBorder
            onClick={() => navigate(`/video/${uuid}`)}>
            <Card.Section>
                <Cover cover={video.cover} width={320} height={180} />
            </Card.Section>

            <Text fw={700}>{title}</Text>
            <Text>直播时间: {date_stamp(stream_date)}</Text>
        </Card >
    )
}

const VideoListView: FC<{ room_id: string }> = ({ room_id }) => {
    const [loading, setLoading] = useState(false)
    const [error, setError] = useState<APIError | null>(null)
    const [list, setList] = useState<VideoList>([])
    const [exhausted, setExhausted] = useState(false)

    console.log(list)

    const loadMore = useCallback(async () => {
        if (loading) return

        setLoading(true)
        setError(null)
        try {
            const result = await requestAPI(VideoList,
                `/api/room/${room_id}/video?offset=${list.length}`)
            if (result.length === 0) {
                setExhausted(true)
            } else {
                setList(v => [...v, ...result])
            }
        } catch (e) {
            if (e instanceof APIError) {
                setError(e)
            } else {
                throw e
            }
        } finally {
            setLoading(false)
        }
    }, [list, loading])

    if (error) {
        return <ErrorPage error={error} />
    }

    return (
        <Flex
            gap="md"
            wrap="wrap"
        >
            {list.map(v => <VideoEntry key={v.uuid} video={v} />)}
            {!exhausted && <VideoLoader list={list} onLoad={loadMore} />}
        </Flex>
    )
}

const Room: FC = () => {
    const params = useParams()
    const room_id = params.room!
    const [asideOpened, { toggle }] = useDisclosure()

    const { loading, result: roomInfo, error } = useAPI(schemas.Room, `/api/room/${room_id}`)

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
                        visible={loading}>
                        <Image
                            w={192}
                            h={192}
                            src={roomInfo?.image ?? phImg(192, 192, "No Image")}
                            referrerPolicy="no-referrer"
                        />
                    </Skeleton>
                    <Skeleton visible={loading}>
                        <Text ta="center" fw={700}>{roomInfo?.username ?? "LOADING"}</Text>
                    </Skeleton>
                    <Skeleton visible={loading}>
                        <Text size="sm" c="dimmed" ta="center">
                            {roomInfo?.short_id ?? roomInfo?.id ?? "114514"}
                        </Text>
                    </Skeleton>
                </Stack>
            </AppShell.Aside>
            <AppShell.Main>
                {error === null ? <VideoListView room_id={room_id} /> : <ErrorPage error={error} />}
            </AppShell.Main>
        </AppShell>
    )
}

export default Room
