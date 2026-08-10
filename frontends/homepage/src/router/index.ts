import { createRouter, createWebHistory } from "vue-router";
import SiteLayout from "../layouts/SiteLayout.vue";
import LandingView from "../views/LandingView.vue";
import DocsView from "../views/DocsView.vue";
import PlaygroundView from "../views/PlaygroundView.vue";
import { defaultDocPath } from "../content/docs";

export const router = createRouter({
    history: createWebHistory(),
    routes: [
        {
            path: "/",
            component: SiteLayout,
            children: [
                {
                    path: "",
                    name: "landing",
                    component: LandingView,
                },
                {
                    path: "playground",
                    name: "playground",
                    component: PlaygroundView,
                },
                {
                    path: "d",
                    redirect: `/d/${defaultDocPath}`,
                },
                {
                    path: "d/:pathMatch(.*)*",
                    name: "docs",
                    component: DocsView,
                    props: true,
                },
                {
                    path: "docs/:pathMatch(.*)*",
                    redirect: (to) => {
                        const rest = to.params.pathMatch;
                        const suffix = Array.isArray(rest)
                            ? rest.join("/")
                            : rest || defaultDocPath;
                        return `/d/${suffix}`;
                    },
                },
                {
                    path: "docs",
                    redirect: `/d/${defaultDocPath}`,
                },
            ],
        },
    ],
    scrollBehavior(to, _from, saved) {
        if (saved) {
            return saved;
        }
        if (to.hash) {
            return { el: to.hash, behavior: "smooth" };
        }
        return { top: 0 };
    },
});
