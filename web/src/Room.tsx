import { FC } from 'react'
import { useDisclosure } from '@mantine/hooks'
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
import { useAPI } from '@lib/api'
import { schemas } from '@lib/schemas'
import ErrorPage from '@components/Error'
import { date_stamp } from '@lib/chrono'

const VideoListEntry = v.omit(schemas.Video, ["room"])
const VideoList = v.array(VideoListEntry)

function phImg(width: number, height: number, text: string) {
    return `https://placehold.co/${width}x${height}?text=${encodeURIComponent(text)}`
}

const VideoEntry: FC<{
    video: v.InferOutput<typeof VideoListEntry>
}> = ({ video }) => {
    const { uuid, title, timestamp } = video

    const date = new Date(timestamp)
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
            <Text>直播时间: {date_stamp(date)}</Text>
        </Card >
    )
}

const VideoListView: FC<{ room_id: string }> = ({ room_id }) => {
    const { loading, result, error } = useAPI(VideoList, `/api/room/${room_id}/video`)
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
            {result!.map(v => <VideoEntry key={v.uuid} video={v} />)}
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
