import type { Component } from 'solid-js';
import { useRoutes, RouteDefinition } from '@solidjs/router';
import Main from './pages/main';
import { lazy } from 'solid-js';

export const routes: RouteDefinition[] = [
    {
        path: '/',
        component: Main,
    },
    {
        path: '**',
        component: lazy(() => import('./errors/404')),
    },
];


const App: Component = () => {
    // const location = useLocation();
    const Route = useRoutes(routes);

    return (
        <>
            <main>
                <Route />
            </main>
        </>
    );
};

export default App;
