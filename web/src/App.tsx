import { Route, Routes } from 'react-router'
import { MantineProvider } from '@mantine/core'

import '@mantine/core/styles.css';

import Home from './Home'
import Room from './Room'
import Video from './Video'

function App() {
    return (
        <MantineProvider>
            <Routes>
                <Route path="/" element={<Home />} />
                <Route path="/room/:room" element={<Room />} />
                <Route path="/video/:video" element={<Video />} />
            </Routes>
        </MantineProvider>
    )
}

export default App
