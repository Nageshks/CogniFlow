import { createRouter, createWebHashHistory } from 'vue-router'

const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    {
      path: '/',
      redirect: '/library',
    },
    {
      path: '/library',
      name: 'library',
      component: () => import('@/views/LibraryView.vue'),
    },
    {
      path: '/workflows',
      name: 'workflows',
      component: () => import('@/views/WorkflowsView.vue'),
    },
    {
      path: '/workflows/:id',
      name: 'workflow-editor',
      component: () => import('@/views/WorkflowEditorView.vue'),
      props: true,
    },

    {
      path: '/settings',
      name: 'settings',
      component: () => import('@/views/SettingsView.vue'),
    },
  ],
})

export default router
