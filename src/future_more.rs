//enum State {
//    // Currently resolving the host name
//    Resolving(ResolveFuture),
//
//    // Establishing a TCP connection to the remote host
//    Connecting(ConnectFuture),
//}
//
//pub struct ResolveAndConnect {
//    state: State,
//}
//
//pub fn resolve_and_connect(host: &str) -> ResolveAndConnect {
//    let state = State::Resolving(resolve(host));
//    ResolveAndConnect { state }
//}
//
//impl Future for ResolveAndConnect {
//    type Item = TcpStream;
//    type Error = io::Error;
//
//    fn poll(&mut self) -> Result<Async<TcpStream>, io::Error> {
//        use self::State::*;
//
//        loop {
//            let addr = match self.state {
//                Resolving(ref mut fut) => {
//                    try_ready!(fut.poll())
//                }
//                Connecting(ref mut fut) => {
//                    return fut.poll();
//                }
//            };
//
//            // If we reach here, the state was `Resolving`
//            // and the call to the inner Future returned `Ready`
//            let connecting = TcpStream::connect(&addr);
//            self.state = Connecting(connecting);
//        }
//    }
//}
//
//resolve(my_host)
//.and_then(|addr| TcpStream::connect(&addr))