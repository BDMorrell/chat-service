export interface User {
    username: string,
};

export interface Message {
    text: string,
    timestamp: Date,
    submitter: User
};
