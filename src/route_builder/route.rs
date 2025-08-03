use crate::route_builder::{RouteAuth, RouteBasic, RoutePublic, RouteRest, RouteUpload};

pub enum Route {
    Auth(RouteAuth),
    Basic(RouteBasic),
    Public(RoutePublic),
    Rest(RouteRest),
    Upload(RouteUpload),
}
