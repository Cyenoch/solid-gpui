/// <reference types="vite/client" />
import { mountGallery } from "@solid-gpui/gallery/application";
mountGallery(import.meta.hot ? import.meta.url : undefined);
