import {
    createContext,
    createRef,
    FC,
    Ref,
    useCallback,
    useContext,
    useEffect,
    useMemo,
    useRef,
    useState
} from 'react'
import { useParams, Link } from 'react-router';
import {
    Anchor,
    AppShell,
    Button,
    Card,
    Collapse,
    Container,
    Flex,
    Group,
    NavLink,
    Paper,
    PasswordInput,
    ScrollArea,
    Skeleton,
    Stack,
    Table,
    Tabs,
    Text,
    TextInput,
    Title,
    useMatches,
} from '@mantine/core'
import { useDisclosure } from '@mantine/hooks'
import { MagnifyingGlassIcon } from '@radix-ui/react-icons'
import * as v from 'valibot'

import SiteTitle from '@components/SiteTitle'
import { requestAPI, useAPI } from '@lib/api';
import { schemas, SchemaTypes } from '@lib/schemas'
import { getMetadataURL, getRestrictedVideoURL, getUnrestrictedVideoURL } from '@lib/objects';
import ErrorPage from '@components/Error';
import { restricted_hash } from '@lib/cryptography';
import { date_stamp } from '@lib/chrono';
import Cover from '@components/Cover';

const PartsListSchema = v.array(v.omit(schemas.Video, ["room"]))

const PartsList: FC<{ uuid: string }> = ({ uuid }) => {
    const { loading, result } = useAPI(PartsListSchema, `/api/video/${uuid}/parts`)

    if (loading) {
        return (
            <NavLink>
                <Skeleton>VOD Title</ Skeleton>
            </NavLink>
        )
    }

    return (
        <ScrollArea>
            {result!.map((e, i) => (
                <NavLink
                    key={e.uuid}
                    active={e.uuid == uuid}
                    to={`/video/${e.uuid}`}
                    component={Link}
                    label={`${i + 1}. ${date_stamp(new Date(e.record_time))}`} />
            ))}
        </ScrollArea>
    )
}

const VODInfo: FC<{ info: SchemaTypes.Video | null }> = ({ info }) => {
    const title = info?.title ?? "PLACEHOLDER"
    const datetime = new Date(info?.stream_time ?? 0)

    const [opened, { toggle }] = useDisclosure(true)

    return (
        <Paper withBorder>
            <Container py="xs" style={{
                cursor: "pointer"
            }} onClick={toggle}>
                <Skeleton visible={info === null}>
                    <Title order={4}>{title}</Title>
                </Skeleton>
            </Container>

            <Collapse in={opened}>
                <Skeleton visible={info === null}>
                    <Cover cover={info?.cover ?? null} width={320} height={160} />
                </Skeleton>

                <Stack p="sm">
                    <Text size="sm" c="dimmed">{date_stamp(datetime)}</Text>
                </Stack>
            </Collapse>
        </Paper >
    )
}

interface ChatGift {
    id: number
    type: "gift"
    timestamp: number
    uid: number
    username: string
    gift_name: string
    count: number
}

interface ChatSub {
    id: number
    type: "sub"
    timestamp: number
    uid: number
    username: string
    sub_name: string
    count: number
}

interface ChatSC {
    id: number
    type: "sc"
    timestamp: number
    uid: number
    username: string
    content: string
    price: number
}

interface ChatMessage {
    id: number
    type: "message"
    timestamp: number
    uid: number
    username: string
    content: string
}

type ChatEntry = ChatGift | ChatSub | ChatMessage | ChatSC

const ChatMessageEntry: FC<{ msg: ChatMessage }> = ({ msg }) => {
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
            return <ChatMessageEntry msg={entry} />
        case "gift":
            return <ChatGiftEntry gift={entry} />
        case "sub":
            return <ChatSubEntry sub={entry} />
        case "sc":
            return <ChatSCEntry sc={entry} />
    }
}

// Binary search to find the first entries after current playback position
// `entries` must be sorted
function binary_search_entries(entries: ChatEntry[], playback_position: number) {
    if (!entries.length) return 0

    if (entries[0].timestamp > playback_position) {
        return 0
    } else if (entries[entries.length - 1].timestamp <= playback_position) {
        return entries.length
    }

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
            <Flex direction="column" gap="md" p="md">
                {display_messages.map(msg => (
                    <ChatEntry key={msg.id} entry={msg} />
                ))}
            </Flex>
        </ScrollArea>
    )
}

function format_ts(ts: number) {
    ts = Math.floor(ts)
    const s = ts % 60
    ts = Math.floor(ts / 60)
    const m = ts % 60
    ts = Math.floor(ts / 60)

    const s_s = s.toString().padStart(2, '0')
    const m_s = m.toString().padStart(2, '0')

    return `${ts.toString()}:${m_s}:${s_s}`
}

const ChatTable: FC<{
    entries: ChatMessage[]
    onSeek?: (pos: number) => void
}> = ({ entries, onSeek }) => {
    return (
        <ScrollArea flex="1 1 0">
            <Table stickyHeader layout="fixed" styles={{
                td: {
                    wordWrap: "break-word"
                }
            }}>
                <Table.Thead>
                    <Table.Tr>
                        <Table.Td w="25%">时间</Table.Td>
                        <Table.Td w="25%">名称</Table.Td>
                        <Table.Td w="50%">内容</Table.Td>
                    </Table.Tr>
                </Table.Thead>

                <Table.Tbody>
                    {entries.map(e => (
                        <Table.Tr key={e.id}>
                            <Table.Td>
                                <Anchor onClick={
                                    () => onSeek && onSeek(e.timestamp)
                                }>
                                    {format_ts(e.timestamp)}
                                </Anchor>
                            </Table.Td>
                            <Table.Td>{e.username}</Table.Td>
                            <Table.Td>{e.content}</Table.Td>
                        </Table.Tr>
                    ))}
                </Table.Tbody>
            </Table>
        </ScrollArea >
    )
}

function parseDTag(item: Element, id: number): ChatMessage {
    const timestamp = parseInt(item.getAttribute("p")!.split(",")[0], 10)
    const uid = parseInt(item.getAttribute("uid")!, 10)
    const username = item!.getAttribute("user")!
    const content = item!.textContent!
    return { id, type: "message", timestamp, uid, username, content }
}

function parseToastTag(item: Element, id: number): ChatSub {
    const timestamp = parseInt(item.getAttribute("ts")!, 10)
    const uid = parseInt(item.getAttribute("uid")!, 10)
    const username = item.getAttribute("user")!
    const sub_name = item.getAttribute("role")!
    const count = parseInt(item.getAttribute("count")!, 10)
    return { id, type: "sub", timestamp, uid, username, sub_name, count }
}

function parseGiftTag(item: Element, id: number): ChatGift {
    const timestamp = parseInt(item.getAttribute("ts")!, 10)
    const uid = parseInt(item.getAttribute("uid")!, 10)
    const username = item.getAttribute("user")!
    const gift_name = item.getAttribute("giftname")!
    const count = parseInt(item.getAttribute("count")!, 10)
    return { id, type: "gift", timestamp, uid, username, gift_name, count }
}

function parseSCTag(item: Element, id: number): ChatSC {
    const timestamp = parseInt(item.getAttribute("ts")!, 10)
    const uid = parseInt(item.getAttribute("uid")!, 10)
    const username = item.getAttribute("user")!
    const content = item.textContent!;
    const price = parseInt(item.getAttribute("price")!, 10)
    return { id, type: "sc", timestamp, uid, username, content, price }
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


const VideoPlayerContext = createContext({
    ref: createRef<HTMLVideoElement>()
})

function filter_by_search(messages: ChatMessage[], query: string) {
    const queries = query.split(" ")
    let ret = messages
    for (let q of queries) {
        if (q.startsWith("uid:")) {
            const uid = parseInt(q.replace("uid:", ""))
            ret = ret.filter(e => e.uid == uid)
        } else if (q.startsWith("uname:")) {
            const uname = q.replace("uname:", "")
            if (!uname) {
                // wait for more input
                return []
            }
            ret = ret.filter(e => e.username.startsWith(uname))
        } else {
            ret = ret.filter(e => e.content.includes(q))
        }
    }
    return ret
}

const ChatList: FC<{
    entries: ChatEntry[],
}> = ({ entries }) => {
    const [searchQuery, setSearchQuery] = useState("")
    const messages = useMemo(() => {
        const ret = entries
            .filter(e => e.type == "message")
        return filter_by_search(ret, searchQuery.trim())
    }, [entries, searchQuery])

    const { ref } = useContext(VideoPlayerContext)

    return (
        <Stack>
            <TextInput value={searchQuery}
                leftSection={<MagnifyingGlassIcon />}
                placeholder="搜索..."
                onChange={v => setSearchQuery(v.currentTarget.value)} />
            <ChatTable entries={messages} onSeek={v => {
                if (!ref.current) return
                ref.current.currentTime = v
            }} />
        </Stack>
    )
}

const SideInfo: FC<{
    info: SchemaTypes.Video | null
    chats: ChatEntry[]
    playbackPosition: number
}> = ({ info, chats, playbackPosition }) => {
    return (
        <Stack flex={{ base: 1, sm: 0 }} miw={{ sm: "400px" }}>
            <VODInfo info={info} />
            <Tabs defaultValue="chat-display" styles={{
                root: {
                    flex: "1 1 0",
                    display: "flex",
                    flexDirection: "column",
                },
                list: {
                    flex: "0",
                },
                panel: {
                    flex: "1 1 0",
                    display: "flex",
                    minHeight: 0,
                }
            }}>
                <Tabs.List>
                    <Tabs.Tab value="chat-display">
                        实时弹幕重放
                    </Tabs.Tab>
                    <Tabs.Tab value="parts">
                        分P
                    </Tabs.Tab>
                    <Tabs.Tab value="chat-list">
                        全部弹幕列表
                    </Tabs.Tab>
                </Tabs.List>

                <Tabs.Panel value="chat-display">
                    <ChatDisplay playbackPosition={playbackPosition} entries={chats} />
                </Tabs.Panel>

                <Tabs.Panel value="parts">
                    {info && <PartsList uuid={info.uuid} />}
                </Tabs.Panel>

                <Tabs.Panel value="chat-list">
                    <ChatList entries={chats} />
                </Tabs.Panel>
            </Tabs>
        </Stack>
    )
}

const VideoPlayer: FC<{
    src: string
    ref?: Ref<HTMLVideoElement>
    onTimeUpdate?: React.ReactEventHandler<HTMLVideoElement>
}> = ({ src, ref, onTimeUpdate }) => {
    const flex = useMatches({
        base: 0,
        sm: 1
    })

    const minHeight = useMatches({
        sm: 0,
    })

    return (
        <video
            controls
            style={{
                flex: flex,
                minWidth: 0,
                minHeight: minHeight,
                objectFit: "contain",
                backgroundColor: "black"
            }}
            ref={ref}
            onTimeUpdate={onTimeUpdate}>
            <source src={src} />
        </video>
    )
}

const VideoView: FC<{
    video: SchemaTypes.Video | null
    source?: string
}> = ({ video, source }) => {
    const [playbackPosition, setPlaybackPosition] = useState(0)
    const [chats, setChats] = useState<ChatEntry[]>([])

    const videoRef = useRef<HTMLVideoElement>(null)

    useEffect(() => {
        const loader = async () => {
            if (!video) return

            const entries = await loadMessages(video.uuid)
            setChats(entries)
        }
        loader()
    }, [video])

    return (
        <VideoPlayerContext.Provider value={{ ref: videoRef }}>
            <Flex
                flex="1 1 0"
                miw={0}
                mih={0}
                direction={{ base: "column", sm: "row" }}>
                {source && (
                    <VideoPlayer
                        ref={videoRef}
                        src={source}
                        onTimeUpdate={e => {
                            setPlaybackPosition(e.currentTarget.currentTime)
                        }}
                    />)}
                <SideInfo info={video} chats={chats}
                    playbackPosition={playbackPosition} />
            </Flex>
        </VideoPlayerContext.Provider>
    )
}

const VerifyPasswordResponse = v.variant('restricted', [
    v.object({
        restricted: v.literal(0)
    }),
    v.object({
        restricted: v.literal(1),
        hash_verified: v.boolean()
    })
])

async function verifyHash(uuid: string, hash: string) {
    const ret = await requestAPI(
        VerifyPasswordResponse,
        `/api/video/${uuid}/restricted?verify_hash=${encodeURIComponent(hash)}`
    )

    if (!ret.restricted) {
        return true
    }
    return ret.hash_verified
}

const RestrictedView: FC<{
    video: SchemaTypes.Video | null,
}> = ({ video }) => {
    const [loading, setLoading] = useState(false)
    const [verified, setVerified] = useState<boolean | null>(null)
    const [password, setPassword] = useState("")
    const [hash, setHash] = useState<string | null>(null)
    const [error, setError] = useState("")
    const inputRef = useRef<HTMLInputElement>(null)

    const onVerify = useCallback(async () => {
        if (!video) return
        if (!password) return

        setLoading(true)
        setVerified(null)
        setHash(null)
        setError("")
        try {
            const hash = await restricted_hash(video.uuid.toLowerCase(), password)
            const result = await verifyHash(video.uuid, hash)
            setVerified(result)
            if (result) {
                setHash(hash)
            }
        } catch (e) {
            if (e instanceof Error) {
                setError(e.message)
            } else {
                setError("unknown error")
            }
        } finally {
            setLoading(false)
        }
    }, [password, video?.uuid])

    useEffect(() => {
        onVerify()
    }, [password, video?.uuid])

    if (!video) {
        return <VideoView video={video} />
    } else if (!video.restricted) {
        const source = getUnrestrictedVideoURL(video)
        return <VideoView video={video} source={source} />
    } else if (hash !== null) {
        const source = getRestrictedVideoURL(video, hash)
        return <VideoView video={video} source={source} />
    }

    return (
        <Container size="md">
            <Stack ta="center" justify="center" py="xl">
                <Title order={1}>/// ACCESS RESTRICTED ///</Title>
                <Text>本视频已被设置为非公开。</Text>
                <Text>如需访问，请在以下提示框中输入口令：</Text>
                <Stack>
                    <Text fw={700}>黑月是否嚎叫？</Text>
                    <PasswordInput
                        ref={inputRef}
                        onKeyUp={e => {
                            if (e.key === "Enter") {
                                e.preventDefault()
                                onVerify()
                            }
                        }}
                        disabled={loading}
                        error={error || (verified === false && "密码错误")}
                    />
                    <Button
                        onClick={() => inputRef.current && setPassword(inputRef.current.value)}>
                        确认
                    </Button>
                </Stack>
            </Stack>
        </Container>
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
                <Group h="100%" px="md" justify="space-between">
                    <SiteTitle />
                </Group>
            </AppShell.Header>
            <AppShell.Main>
                {error === null ? (
                    <RestrictedView video={video} />
                ) : (
                    <ErrorPage error={error} />
                )}
            </AppShell.Main>
        </AppShell >
    )
}

export default Video
