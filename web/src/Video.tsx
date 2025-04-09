import { FC, useEffect, useState } from 'react'
import { useParams } from 'react-router';
import { AppShell, Flex, Group } from '@mantine/core'

import SiteTitle from './components/SiteTitle';

const video_base_url = import.meta.env.VITE_KOISHI_CDN_PREFIX;

interface VideoInfo {
    uuid: string,
    title: string,
    room: number
}

const Video: FC = () => {
    const [video, setVideo] = useState<VideoInfo | null>(null)

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

    const source = video && new URL(`/video/${video.room}/${video.uuid}`, video_base_url)

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
                    flex={{ base: 1 }}
                    direction={{ base: "column", sm: "row" }}>
                    {
                        source && (
                            <video controls style={{
                                flex: 1,
                                width: '100%',
                                objectFit: "contain",
                                backgroundColor: "black"
                            }}>
                                <source src={source.toString()} />
                            </video>
                        )
                    }
                    <div />
                </Flex>
            </AppShell.Main>
        </AppShell>
    )
}

export default Video
