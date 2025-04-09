import { FC, useEffect, useState } from 'react'
import { useParams } from 'react-router';

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

    const source = video && new URL(`/${video.room}/${video.uuid}`, video_base_url)

    return (
        <>
            {source && (
                <video controls style={{ width: "100%", height: "!00%" }}>
                    <source src={source.toString()} />
                </video>
            )}
        </>
    )
}

export default Video
