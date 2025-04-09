import { FC, useEffect, useMemo, useRef, useState } from 'react'
import { useParams } from 'react-router';
import { AppShell, Card, Flex, Group, ScrollArea, Skeleton, Stack, Text } from '@mantine/core'

import SiteTitle from './components/SiteTitle';

const cdn_base_url = import.meta.env.VITE_KOISHI_CDN_PREFIX;

interface VideoInfo {
    uuid: string,
    title: string,
    room: number
    timestamp: number
}

const VODInfo: FC<{ info: VideoInfo | null }> = ({ info }) => {
    const title = info?.title ?? "PLACEHOLDER"
    const datetime = new Date(info?.timestamp ?? 0)

    return (
        <Card
            shadow="sm"
            padding="xl"
        >
            <Stack>
                <Skeleton visible={info === null}>
                    <Text fw={700}>{title}</Text>
                </Skeleton>
                <Skeleton visible={info === null}>
                    <Text size="sm" c="dimmed">{datetime.toDateString()}</Text>
                </Skeleton>
            </Stack>
        </Card>
    )
}

interface ChatMsg {
    id: number,
    timestamp: number,
    username: string,
    content: string
}

const ChatEntry: FC<{ msg: ChatMsg }> = ({ msg }) => {
    return (
        <Group gap="1">
            <Text fw={700}>{msg.username}:</Text>
            <Text>{msg.content}</Text>
        </Group>
    )
}

// Binary search to find the newest message until current playback position
// `msgs` must be sorted
function binary_search_msg(msgs: ChatMsg[], playback_position: number) {
    let lower = 0
    let upper = msgs.length
    while (true) {
        let middle = Math.floor((upper - lower) / 2 + lower)
        if (middle == lower || middle == upper) {
            return upper
        }

        if (msgs[middle].timestamp <= playback_position) {
            lower = middle
        } else {
            upper = middle
        }
    }
}

const ChatDisplay: FC<{
    messages: ChatMsg[]
    playbackPosition: number
}> = ({ messages, playbackPosition }) => {
    const following = true	// TODO

    const viewport = useRef<HTMLDivElement>(null)

    useEffect(() => {
        if (!following) return
        let id = setInterval(() => {
        }, 500)
        return () => {
            clearInterval(id)
        }
    }, [following])

    const sorted_messages = useMemo(() => {
        return messages.sort((a, b) => a.timestamp - b.timestamp)
    }, [messages])

    const [latest_idx, display_messages] = useMemo(() => {
        if (!following) {
            return [0, sorted_messages]
        }

        const latest_idx = binary_search_msg(sorted_messages, playbackPosition)
        return [latest_idx, messages.slice(Math.max(latest_idx - 100, 0), latest_idx)]
    }, [sorted_messages, following, playbackPosition])

    useEffect(() => {
        if (!following) return
        viewport.current?.scrollTo({
            top: viewport.current.scrollHeight,
            behavior: "smooth"
        })
    }, [latest_idx])

    return (
        <ScrollArea style={{ flex: "1 1 0", overflowY: "auto" }} viewportRef={viewport}>
            <Flex direction="column" gap="md">
                {display_messages.map(msg => (
                    <ChatEntry key={msg.id} msg={msg} />
                ))}
            </Flex>
        </ScrollArea>
    )
}

const SideInfo: FC<{
    info: VideoInfo | null
    playbackPosition: number
}> = ({ info, playbackPosition }) => {
    const [messages, setMessages] = useState<ChatMsg[]>([])

    useEffect(() => {
        const loader = async () => {
            if (!info) return

            const metadata_url = new URL(`/metadata/${info.uuid}`, cdn_base_url)
            const res = await fetch(metadata_url)
            if (res.status == 404) {
                return
            }
            const xml = await res.text()
            const parser = new DOMParser()
            const doc = parser.parseFromString(xml, "text/xml")

            const msgs: ChatMsg[] = []
            const tags = doc.getElementsByTagName("d")
            for (let i = 0; i < tags.length; ++i) {
                const item = tags.item(i)
                const timestamp = parseInt(item!.getAttribute("p")!.split(",")[0], 10)
                const username = item!.getAttribute("user")!
                const content = item!.textContent!
                msgs.push({ id: i, timestamp, username, content })
            }
            setMessages(msgs)
        }
        loader()
    }, [info])

    return (
        <Stack flex={{ base: 1, sm: 0 }} miw={{ sm: "300px" }} p="xl">
            <VODInfo info={info} />
            <ChatDisplay playbackPosition={playbackPosition} messages={messages} />
        </Stack>
    )
}

const Video: FC = () => {
    const [video, setVideo] = useState<VideoInfo | null>(null)
    const [playbackPosition, setPlaybackPosition] = useState(0)

    const params = useParams()
    const video_id = params.video!

    useEffect(() => {
        const loader = async () => {
            const res = await fetch(`/api/video/${video_id}`)
            const body = await res.json()

            setVideo(body.data)
        }
        loader()
    }, [video_id])

    const source = video && new URL(`/video/${video.room}/${video.uuid}`, cdn_base_url)

    return (
        <AppShell
            header={{ height: 60 }}
            styles={{
                main: {
                    display: 'flex',
                    flexDirection: 'column'
                }
            }}>
            <AppShell.Header>
                <Group h="100%" px="md">
                    <SiteTitle />
                </Group>
            </AppShell.Header>
            <AppShell.Main>
                <Flex
                    flex={1}
                    direction={{ base: "column", sm: "row" }}>
                    {
                        source && (
                            <video controls style={{
                                flex: 1,
                                minWidth: 0,
                                objectFit: "contain",
                                backgroundColor: "black"
                            }}
                                onTimeUpdate={e => setPlaybackPosition(
                                    e.currentTarget.currentTime
                                )}>
                                <source src={source.toString()} />
                            </video>
                        )
                    }
                    <SideInfo info={video} playbackPosition={playbackPosition} />
                </Flex>
            </AppShell.Main>
        </AppShell >
    )
}

export default Video
