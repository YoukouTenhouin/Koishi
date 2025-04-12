import { FC } from 'react'
import { Image } from '@mantine/core'

import { getCoverURL } from '@lib/objects'

function phImg(width: number, height: number, text: string) {
    return `https://placehold.co/${width}x${height}?text=${encodeURIComponent(text)}`
}

const Cover: FC<{
    width?: number,
    height?: number,
    cover: string | null
}> = ({ cover, width, height }) => {
    const url = cover && getCoverURL(cover)
    return (
        <Image
            width={width}
            height={height}
            src={url ?? phImg(320, 180, "NO COVER")}
            referrerPolicy="no-referrer" />
    )
}

export default Cover
