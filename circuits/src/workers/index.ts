export type MessageKind = 'init' | 'job'

export interface WorkerInitMessage<T> {
    kind: 'init'
    args: T
}

export interface WorkerJobMessage<T> {
    kind: 'job'
    id: number
    data: T
}

export type WorkerMessage<InitArgs, JobData> = WorkerInitMessage<InitArgs> | WorkerJobMessage<JobData>

export interface WorkerInitResponse {
    kind: 'init'
    error?: Error
}

export interface WorkerJobResponse<Result> {
    kind: 'job'
    id: number
    result?: Result
    error?: Error
}


export type WorkerResponse<Result> = WorkerInitResponse | WorkerJobResponse<Result>

interface Callback<T> {
    resolve: (result: T) => void,
    reject: (error: Error) => void
}

export class WorkerDispatcher<InitArgs, JobData, Result> {
    private worker: Worker
    private poisoned = false
    private initialized = false
    private initializeCallback: Callback<void> | undefined
    private jobCounter = 0
    private callbacks: Map<number, Callback<Result>> = new Map()

    constructor(worker: Worker) {
        this.worker = worker
        this.worker.onerror = (error) => {
            if (error.message) {
                console.error(`Worker Error ${error.message}`)
            } else {
                console.error('Worker Error', error)
            }
            this.onerror()
        }
        this.worker.onmessage = ({ data }: MessageEvent<WorkerResponse<Result>>) => {
            if (data.kind === 'init') {
                console.assert(!this.initialized, 'Should not call init twice')
                console.assert(this.initializeCallback !== undefined, 'Should have initializeCallback')
                if (data.error) {
                    this.onerror()
                    this.initializeCallback!.reject(data.error)
                } else {
                    this.initialized = true
                    this.initializeCallback!.resolve()
                }
                this.initializeCallback = undefined
            } else if (data.kind === 'job') {
                const { id, result, error } = data
                const callback = this.callbacks.get(id)
                if (callback) {
                    if (error) {
                        callback.reject(error)
                    } else {
                        callback.resolve(result!)
                    }
                    this.callbacks.delete(id)
                }
            }
        }
    }

    onerror() {
        this.poisoned = true
        this.worker.onerror = null
        this.worker.onmessage = null
        this.worker.terminate()
        this.callbacks.forEach(({ reject }) => reject(new Error('Worker is poisoned')))
        this.callbacks.clear()
    }

    public async init(args: InitArgs): Promise<void> {
        if (this.poisoned) {
            throw new Error('Worker is poisoned')
        }
        if (this.initialized) {
            throw new Error('Worker is already initialized')
        }
        const promise = new Promise<void>((resolve, reject) => {
            this.initializeCallback = { resolve, reject }
        })
        this.worker.postMessage({ kind: 'init', args } as WorkerInitMessage<InitArgs>)
        return promise
    }

    public async postMessage(data: JobData): Promise<Result> {
        if (this.poisoned) {
            throw new Error('Worker is poisoned')
        }
        const id = this.jobCounter++
        const promise = new Promise<Result>((resolve, reject) => {
            this.callbacks.set(id, {resolve, reject})
        })
        this.worker.postMessage({ kind: 'job', id, data } as WorkerJobMessage<JobData>)
        return promise
    }
}