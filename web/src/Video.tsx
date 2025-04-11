import { FC, useEffect, useMemo, useRef, useState } from 'react'
import { useParams } from 'react-router';
import {
    AppShell,
    Card,
    Flex,
    Group,
    ScrollArea,
    Skeleton,
    Stack,
    Text,
    useMatches
} from '@mantine/core'

import SiteTitle from '@components/SiteTitle'
import { schemas, SchemaTypes } from '@lib/schemas'
import { getMetadataURL, getVideoURL } from '@lib/objects';
import { useAPI } from '@lib/api';
import ErrorPage from '@components/Error';

const VODInfo: FC<{ info: SchemaTypes.Video | null }> = ({ info }) => {
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

interface ChatGift {
    id: number
    type: "gift"
    timestamp: number
    username: string
    gift_name: string
    count: number
}

interface ChatSub {
    id: number
    type: "sub"
    timestamp: number
    username: string
    sub_name: string
    count: number
}

interface ChatSC {
    id: number
    type: "sc"
    timestamp: number
    username: string
    content: string
    price: number
}

interface ChatMsg {
    id: number,
    type: "message"
    timestamp: number,
    username: string,
    content: string
}

type ChatEntry = ChatGift | ChatSub | ChatMsg | ChatSC

const ChatMsgEntry: FC<{ msg: ChatMsg }> = ({ msg }) => {
    return (
        <Group gap="1">
            <Text fw={700}>{msg.username}:</Text>
            <Text>{msg.content}</Text>
        </Group>
    )
}

const ChatGiftEntry: FC<{ gift: ChatGift }> = ({ gift }) => {
    return (
        <Card shadow="xs" withBorder p="sm" bg="orange">
            <Group gap="sm" justify="center">
                <Text fw={700}>{gift.username}</Text>
                <Text>赠送</Text>
                <Text fw={700}>{gift.gift_name}</Text>
                {gift.count > 1 && <Text>×{gift.count}</Text>}
            </Group>
        </Card>
    )
}

const ChatSubEntry: FC<{ sub: ChatSub }> = ({ sub }) => {
    return (
        <Card shadow="xs" withBorder p="sm" bg="blue">
            <Group gap="sm" justify="center">
                <Text fw={700}>{sub.username}</Text>
                <Text>续费</Text>
                <Text fw={700}>{sub.sub_name}</Text>
                {sub.count > 1 && <Text>×{sub.count}</Text>}
            </Group>
        </Card>
    )
}

const ChatSCEntry: FC<{ sc: ChatSC }> = ({ sc }) => {
    return (
        <Card shadow="xs" withBorder p="xl" bg="red">
            <Card.Section>
                <Group gap="sm">
                    <Text>¥{sc.price / 1000}</Text>
                    <Text fw={700}>{sc.username}</Text>
                </Group>
            </Card.Section>

            <Card.Section>
                <Text>{sc.content}</Text>
            </Card.Section>
        </Card>
    )
}

const ChatEntry: FC<{ entry: ChatEntry }> = ({ entry }) => {
    switch (entry.type) {
        case "message":
            return <ChatMsgEntry msg={entry} />
        case "gift":
            return <ChatGiftEntry gift={entry} />
        case "sub":
            return <ChatSubEntry sub={entry} />
        case "sc":
            return <ChatSCEntry sc={entry} />
    }
}

// Binary search to find the newest entries until current playback position
// `entries` must be sorted
function binary_search_entries(entries: ChatEntry[], playback_position: number) {
    let lower = 0
    let upper = entries.length
    while (true) {
        let middle = Math.floor((upper - lower) / 2 + lower)
        if (middle == lower || middle == upper) {
            return upper
        }

        if (entries[middle].timestamp <= playback_position) {
            lower = middle
        } else {
            upper = middle
        }
    }
}

const ChatDisplay: FC<{
    entries: ChatEntry[]
    playbackPosition: number
}> = ({ entries, playbackPosition }) => {
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

    const sorted_entries = useMemo(() => {
        return entries.sort((a, b) => a.timestamp - b.timestamp)
    }, [entries])

    const [latest_idx, display_messages] = useMemo(() => {
        if (!following) {
            return [0, sorted_entries]
        }

        const latest_idx = binary_search_entries(sorted_entries, playbackPosition)
        return [latest_idx, entries.slice(Math.max(latest_idx - 100, 0), latest_idx)]
    }, [sorted_entries, following, playbackPosition])

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
                    <ChatEntry key={msg.id} entry={msg} />
                ))}
            </Flex>
        </ScrollArea>
    )
}

function parseDTag(item: Element, id: number): ChatMsg {
    const timestamp = parseInt(item.getAttribute("p")!.split(",")[0], 10)
    const username = item!.getAttribute("user")!
    const content = item!.textContent!
    return { id, type: "message", timestamp, username, content }
}

function parseToastTag(item: Element, id: number): ChatSub {
    const timestamp = parseInt(item.getAttribute("ts")!, 10)
    const username = item.getAttribute("user")!
    const sub_name = item.getAttribute("role")!
    const count = parseInt(item.getAttribute("count")!, 10)
    return { id, type: "sub", timestamp, username, sub_name, count }
}

function parseGiftTag(item: Element, id: number): ChatGift {
    const timestamp = parseInt(item.getAttribute("ts")!, 10)
    const username = item.getAttribute("user")!
    const gift_name = item.getAttribute("giftname")!
    const count = parseInt(item.getAttribute("count")!, 10)
    return { id, type: "gift", timestamp, username, gift_name, count }
}

function parseSCTag(item: Element, id: number): ChatSC {
    const timestamp = parseInt(item.getAttribute("ts")!, 10)
    const username = item.getAttribute("user")!
    const content = item.textContent!;
    const price = parseInt(item.getAttribute("price")!, 10)
    return { id, type: "sc", timestamp, username, content, price }
}

async function loadMessages(uuid: string) {
    const metadata_url = getMetadataURL(uuid)
    const res = await fetch(metadata_url)
    if (res.status == 404) {
        return []
    }
    const xml = await res.text()
    const parser = new DOMParser()
    const doc = parser.parseFromString(xml, "text/xml")
    const root = doc.firstElementChild!

    const entries: ChatEntry[] = []
    const tags = root.children
    for (let i = 0; i < tags.length; ++i) {
        const item = tags.item(i)!
        switch (item.tagName) {
            case "d":
                entries.push(parseDTag(item, i))
                break
            case "toast":
                entries.push(parseToastTag(item, i))
                break
            case "gift":
                entries.push(parseGiftTag(item, i))
                break
            case "sc":
                entries.push(parseSCTag(item, i))
                break
            default:
                break
        }
    }

    return entries
}

const SideInfo: FC<{
    info: SchemaTypes.Video | null
    playbackPosition: number
}> = ({ info, playbackPosition }) => {
    const [entries, setEntries] = useState<ChatEntry[]>([])

    useEffect(() => {
        const loader = async () => {
            if (!info) return

            const entries = await loadMessages(info.uuid)
            setEntries(entries)
        }
        loader()
    }, [info])

    return (
        <Stack flex={{ base: 1, sm: 0 }} miw={{ sm: "300px" }} p="xl">
            <VODInfo info={info} />
            <ChatDisplay playbackPosition={playbackPosition} entries={entries} />
        </Stack>
    )
}

const VideoView: FC<{ video: SchemaTypes.Video | null }> = ({ video }) => {
    const [playbackPosition, setPlaybackPosition] = useState(0)

    const videoPlayerFlex = useMatches({
        base: 0,
        sm: 1
    })

    const source = video && getVideoURL(video.room, video.uuid)

    return (
        <Flex
            flex={1}
            direction={{ base: "column", sm: "row" }}>
            {source && (
                <video
                    controls
                    style={{
                        flex: videoPlayerFlex,
                        minWidth: 0,
                        minHeight: 0,
                        objectFit: "contain",
                        backgroundColor: "black"
                    }}
                    onTimeUpdate={e => setPlaybackPosition(
                        e.currentTarget.currentTime
                    )}>
                    <source src={source.toString()} />
                </video>
            )}
            <SideInfo info={video} playbackPosition={playbackPosition} />
        </Flex>
    )
}

const Video: FC = () => {
    const params = useParams()
    const video_id = params.video!

    const { result: video, error } = useAPI(schemas.Video, `/api/video/${video_id}`)


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
                {error === null ? <VideoView video={video} /> : <ErrorPage error={error} />}
            </AppShell.Main>
        </AppShell >
    )
}

export default Video
