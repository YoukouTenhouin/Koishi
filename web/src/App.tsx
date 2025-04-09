import { Route, Routes } from 'react-router';
import { AppShell, MantineProvider } from '@mantine/core';

import '@mantine/core/styles.css';

import Home from './Home'
import Room from './Room'
import Video from './Video'

function App() {
    return (
        <MantineProvider>
            <AppShell>
                <AppShell.Main>
                    <Routes>
                        <Route path="/" element={<Home />} />
                        <Route path="/room/:room" element={<Room />} />
                        <Route path="/video/:video" element={<Video />} />
                    </Routes>
                </AppShell.Main>
            </AppShell>
        </MantineProvider>
    )
}

export default App
