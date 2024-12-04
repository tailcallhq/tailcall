import type { ConnectRouter } from "@connectrpc/connect";
import { NewsService } from "./gen/news_pb";
import { Status } from "./gen/news_dto_pb";

// In-memory storage for news items
const newsStore = new Map([
    [1, {
        id: 1,
        title: "Hello World-1",
        body: "This is a test-1",
        postImage: "https://via.placeholder.com/150",
        status: Status.PUBLISHED,
    }],
    [2, {
        id: 2,
        title: "Hello World-2",
        body: "This is a test-2",
        postImage: "https://via.placeholder.com/150",
        status: Status.PUBLISHED,
    }]
]);

let nextId = 3;

export default (router: ConnectRouter) =>
    router.service(NewsService, {
        async getAllNews(req) {
            return {
                news: Array.from(newsStore.values())
            };
        },

        async getNews(req) {
            const news = newsStore.get(req.id);
            if (!news) {
                throw new Error(`News with id ${req.id} not found`);
            }
            return news;
        },

        async getMultipleNews(req) {
            const newsItems = req.ids
                .map(newsId => newsStore.get(newsId.id))
                .filter((news): news is NonNullable<typeof news> => news !== undefined);
            
            return {
                news: newsItems
            };
        },

        async deleteNews(req) {
            if (!newsStore.has(req.id)) {
                throw new Error(`News with id ${req.id} not found`);
            }
            newsStore.delete(req.id);
            return new Empty();
        },

        async editNews(req) {
            if (!newsStore.has(req.id)) {
                throw new Error(`News with id ${req.id} not found`);
            }
            newsStore.set(req.id, req);
            return req;
        },

        async addNews(req) {
            const id = nextId++;
            const news = {
                ...req,
                id
            };
            newsStore.set(id, news);
            return news;
        }
    });