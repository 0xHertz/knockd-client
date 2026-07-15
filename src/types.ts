export interface Connection {
  id?: number;
  name: string;
  connType: "ssh" | "web";
  host: string;
  port?: number;
  username?: string;
  sshClient?: string;
  knockPorts: string;
  knockProtocol: string;
  knockDelayMs: number;
  launchUri?: string;
  createdAt?: string;
  updatedAt?: string;
}

export interface KnockStep {
  protocol: string;
  port: number;
}

export interface SshClient {
  name: string;
  path: string;
  installed: boolean;
}
