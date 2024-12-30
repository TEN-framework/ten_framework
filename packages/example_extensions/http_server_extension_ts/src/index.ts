//
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0.
// See the LICENSE file for more information.
//

import { Addon, RegisterAddonAsExtension, Extension, TenEnv, Cmd, StatusCode } from 'ten-runtime-nodejs';
// import { Addon, RegisterAddonAsExtension, Extension, TenEnv, Cmd, StatusCode } from '../../../../core/src/ten_runtime/binding/nodejs/interface';
import * as http from 'http';

@RegisterAddonAsExtension("http_server_extension_ts")
class DefaultAddon extends Addon {
    async onInit() {
        console.log('DefaultAddon onInit');
    }

    async onCreateInstance(tenEnv: TenEnv, instanceName: string): Promise<Extension> {
        return new DefaultExtension(instanceName);
    }

    async onDeinit() {
        console.log('DefaultAddon onDeinit');
    }
}

class DefaultExtension extends Extension {
    tenEnvProxy: TenEnv | null;
    httpServer: http.Server;

    constructor(name: string) {
        super(name);
    }

    async onConfigure(tenEnv: TenEnv) {
        console.log('DefaultExtension onConfigure');
    }

    async onInit(tenEnv: TenEnv) {
        this.tenEnvProxy = tenEnv;
        console.log('DefaultExtension onInit');
    }

    async handler(req: http.IncomingMessage, res: http.ServerResponse) {
        if (req.method === 'POST' && req.headers['content-type'] === 'application/json') {
            let body = '';

            req.on('data', chunk => {
                body += chunk;
            });

            req.on('end', () => {
                let jsonData: any;
                try {
                    jsonData = JSON.parse(body);
                    console.log('Parsed JSON:', jsonData);
                } catch (error) {
                    console.error('Error parsing JSON:', error);
                    res.writeHead(400, { 'Content-Type': 'text/plain' });
                    res.end('Error parsing JSON');
                    return;
                }

                // if '_ten' not in the JSON data
                if (!('_ten' in jsonData)) {
                    console.log('No _ten in JSON data');
                    res.writeHead(400, { 'Content-Type': 'text/plain' });
                    res.end('No _ten in JSON data');
                    return;
                }

                const _ten = jsonData['_ten'];
                if ('type' in _ten && _ten['type'] == 'close_app') {
                    let closeAppCmd = Cmd.Create('ten:close_app');
                    closeAppCmd.setDest("localhost");
                    this.tenEnvProxy!.sendCmd(closeAppCmd);

                    res.writeHead(200, { 'Content-Type': 'application/json' });
                    res.end(JSON.stringify({ message: 'OK' }));
                    return;
                } else if ('name' in _ten) {
                    const name = _ten['name'];
                    const cmd = Cmd.Create(name);
                    cmd.setPropertyString('method', req.method!);
                    cmd.setPropertyString('url', req.url!);
                    cmd.setPropertyFromJson('', body);

                    this.tenEnvProxy!.sendCmd(cmd).then(([cmdResult, error]) => {
                        if (error) {
                            res.writeHead(500, { 'Content-Type': 'text/plain' });
                            res.end('Error: ' + error.message);
                        } else {
                            if (cmdResult?.getStatusCode() == StatusCode.OK) {
                                const detail = cmdResult!.getPropertyToJson('detail');
                                res.writeHead(200, { 'Content-Type': 'application/json' });
                                res.end(detail);
                            } else {
                                console.error('Error: ' + cmdResult?.getStatusCode());
                                res.writeHead(500, { 'Content-Type': 'text/plain' });
                                res.end('Internal Server Error');
                            }
                        }
                    });
                } else {
                    console.error('Invalid _ten');
                    res.writeHead(400, { 'Content-Type': 'text/plain' });
                    res.end('Invalid _ten');
                }
            });
        }
    }

    async onStart(tenEnv: TenEnv) {
        console.log('DefaultExtension onStart');

        const hostname = '127.0.0.1';
        var port: number;
        try {
            port = await tenEnv.getPropertyNumber('server_port');
        } catch (e) {
            // Use default port
            port = 8001;
        }

        // Start a simple http server
        const server = http.createServer(this.handler.bind(this));

        server.listen(port, () => {
            console.log('Server running at http://' + hostname + ':' + port + '/');
        });

        this.httpServer = server;
    }

    async onStop(tenEnv: TenEnv) {
        console.log('DefaultExtension onStop');

        await new Promise<void>((resolve, reject) => {
            this.httpServer.close((err) => {
                if (err) {
                    reject(err);
                } else {
                    resolve();
                }
            });
        });
    }

    async onDeinit(tenEnv: TenEnv) {
        this.tenEnvProxy = null;
        console.log('DefaultExtension onDeinit');
    }
}