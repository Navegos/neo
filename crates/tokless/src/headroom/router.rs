use std::sync::Arc;
use crate::util::routing_coordinator::RoutingCoordinator;
use crate::util::routing_plan::{route_protocol_for_path, RouteRequest, RouteSelection, RoutingSnapshot};

pub const ROUTER_ROUTE_HEADER: &str = "X-Tokless-Route";

pub struct Router {
    coordinator: Option<Arc<RoutingCoordinator>>,
    snapshot: Option<RoutingSnapshot>,
}

impl Router {
    pub fn from_coordinator(coordinator: Arc<RoutingCoordinator>) -> Self {
        Self {
            coordinator: Some(coordinator),
            snapshot: None,
        }
    }

    pub fn from_snapshot(snapshot: RoutingSnapshot) -> Self {
        Self {
            coordinator: None,
            snapshot: Some(snapshot),
        }
    }

    pub fn resolve_route_request(
        &self,
        credential: &str,
        path: &str,
        method: &str,
    ) -> Result<RouteSelection, Box<dyn std::error::Error + Send + Sync>> {
        let protocol = route_protocol_for_path(path).ok_or("unsupported route protocol")?;
        let req = RouteRequest {
            credential: credential.to_string(),
            protocol: protocol.to_string(),
            method: method.to_string(),
            path: path.to_string(),
            raw_path: String::new(),
            raw_query: String::new(),
        };

        if let Some(ref coord) = self.coordinator {
            coord.resolve_and_authorize(req)
        } else if let Some(ref snap) = self.snapshot {
            snap.resolve_request(req, |_| true)
        } else {
            Err("router has no active snapshot or coordinator".into())
        }
    }
}

