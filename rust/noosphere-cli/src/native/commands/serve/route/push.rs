use std::sync::Arc;

use anyhow::Result;

use axum::{extract::ContentLengthLimit, http::StatusCode, Extension};

use cid::Cid;
use noosphere::sphere::SphereContext;
use noosphere_api::data::{PushBody, PushResponse};
use noosphere_core::{
    authority::{Authorization, SphereAction, SphereReference},
    data::{Bundle, MapOperation},
    view::{Sphere, SphereMutation, Timeline},
};
use noosphere_ns::DHTKeyMaterial;
use noosphere_storage::{NativeStorage, SphereDb};
use tokio::sync::{mpsc::UnboundedSender, Mutex};
use ucan::capability::{Capability, Resource, With};

use crate::native::commands::serve::{
    authority::GatewayAuthority, extractor::Cbor, gateway::GatewayScope, ipfs::SyndicationJob,
    name_system::NSJob,
};

//use axum::debug_handler;
//#[debug_handler]
pub async fn push_route<K>(
    authority: GatewayAuthority<K>,
    ContentLengthLimit(Cbor(push_body)): ContentLengthLimit<Cbor<PushBody>, { 1024 * 5000 }>,
    Extension(sphere_context_mutex): Extension<Arc<Mutex<SphereContext<K, NativeStorage>>>>,
    Extension(scope): Extension<GatewayScope>,
    Extension(syndication_tx): Extension<UnboundedSender<SyndicationJob<K, NativeStorage>>>,
    Extension(ns_tx): Extension<Option<UnboundedSender<NSJob>>>,
) -> Result<Cbor<PushResponse>, StatusCode>
where
    K: DHTKeyMaterial + 'static,
{
    debug!("Invoking push route...");

    let sphere_identity = &push_body.sphere;

    if sphere_identity != &scope.counterpart {
        return Err(StatusCode::FORBIDDEN);
    }

    authority.try_authorize(&Capability {
        with: With::Resource {
            kind: Resource::Scoped(SphereReference {
                did: scope.counterpart.to_string(),
            }),
        },
        can: SphereAction::Push,
    })?;

    let sphere_context = sphere_context_mutex.lock().await;
    let mut db = sphere_context.db().clone();
    let gateway_key = &sphere_context.author().key;
    let gateway_authorization =
        sphere_context
            .author()
            .require_authorization()
            .map_err(|error| {
                error!("{:?}", error);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

    debug!("Preparing to merge sphere lineage...");
    let local_sphere_base_cid = db.get_version(sphere_identity).await.map_err(|error| {
        error!("{:?}", error);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    let request_sphere_base_cid = push_body.base;

    match (local_sphere_base_cid, request_sphere_base_cid) {
        (Some(mine), theirs) => {
            // TODO(#26): Probably should do some diligence here to check if
            // their base is even in our lineage. Note that this condition
            // will be hit if theirs is ahead of mine, which actually
            // should be a "missing revisions" condition.
            let conflict = match theirs {
                Some(cid) if cid != mine => true,
                None => true,
                _ => false,
            };

            if conflict {
                warn!("Conflict!");
                return Err(StatusCode::CONFLICT);
            }

            if push_body.tip == mine {
                warn!("No new changes in push body!");
                return Ok(Cbor(PushResponse::NoChange));
            }
        }
        (None, Some(_)) => {
            error!("Missing local lineage!");
            return Err(StatusCode::UNPROCESSABLE_ENTITY);
        }
        _ => (),
    };

    debug!("Merging...");

    {
        incorporate_lineage::<K>(&scope, &mut db, &push_body, &ns_tx)
            .await
            .map_err(|error| {
                error!("{:?}", error);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
    }

    debug!("Updating the gateway's sphere...");

    let (new_gateway_tip, new_blocks) = update_gateway_sphere(
        &push_body.tip,
        &scope,
        gateway_key,
        gateway_authorization,
        &mut db,
    )
    .await
    .map_err(|error| {
        error!("{:?}", error);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // TODO(#156): These jobs should not be happening on every push, but rather on
    // an explicit publish action. Move this to the publish handler when we
    // have added it to the gateway.
    if let Err(error) = syndication_tx.send(SyndicationJob {
        revision: new_gateway_tip,
        context: sphere_context_mutex.clone(),
    }) {
        warn!("Failed to queue IPFS syndication job: {}", error);
    };

    Ok(Cbor(PushResponse::Accepted {
        new_tip: new_gateway_tip,
        blocks: new_blocks,
    }))
}

async fn update_gateway_sphere<K>(
    counterpart_sphere_cid: &Cid,
    scope: &GatewayScope,
    key: &K,
    authority: &Authorization,
    db: &mut SphereDb<NativeStorage>,
) -> Result<(Cid, Bundle)>
where
    K: DHTKeyMaterial + 'static,
{
    let my_sphere_cid = db.require_version(&scope.identity).await?;

    let my_sphere = Sphere::at(&my_sphere_cid, db);
    let my_did = key.get_did().await?;

    let mut mutation = SphereMutation::new(&my_did);
    mutation
        .links_mut()
        .set(&scope.counterpart, counterpart_sphere_cid);

    let mut revision = my_sphere.try_apply_mutation(&mutation).await?;

    let my_updated_sphere_cid = revision.try_sign(key, Some(authority)).await?;

    db.set_version(&scope.identity, &my_updated_sphere_cid)
        .await?;

    let blocks = Sphere::at(&my_updated_sphere_cid, db)
        .try_bundle_until_ancestor(Some(&my_sphere_cid))
        .await?;

    Ok((my_updated_sphere_cid, blocks))
}

async fn incorporate_lineage<K>(
    scope: &GatewayScope,
    db: &mut SphereDb<NativeStorage>,
    push_body: &PushBody,
    ns_tx: &Option<UnboundedSender<NSJob>>,
) -> Result<()>
where
    K: DHTKeyMaterial + 'static,
{
    push_body.blocks.load_into(db).await?;

    let PushBody { base, tip, .. } = push_body;

    let timeline = Timeline::new(db);
    let timeslice = timeline.slice(tip, base.as_ref());
    let steps = timeslice.try_to_chronological().await?;
    for (cid, _) in steps {
        info!("Hydrating {:#?}", cid);
        let sphere = Sphere::at(&cid, db);
        sphere.try_hydrate().await?;
        request_ns_addresses(&sphere, ns_tx).await?;
    }
    db.set_version(&scope.counterpart, &push_body.tip).await?;

    Ok(())
}

/// Looks for new entries in a sphere's address book,
/// and makes a request to the Name System to find
/// the latest address.
async fn request_ns_addresses(
    sphere: &Sphere<SphereDb<NativeStorage>>,
    ns_tx: &Option<UnboundedSender<NSJob>>,
) -> Result<()> {
    if ns_tx.is_none() {
        info!("Skipping NS address request, NS not configured");
        return Ok(());
    }

    let names = sphere.try_get_names().await?;
    let changelog = names.try_get_changelog().await?;

    for op in &changelog.changes {
        match op {
            MapOperation::Add {
                key: name,
                value: address,
                ..
            } => {
                if let Err(e) = ns_tx.as_ref().unwrap().send(NSJob::GetRecord {
                    pet_name: name.to_owned(),
                    sphere_id: address.identity.clone(),
                }) {
                    warn!("Failed to queue name system job: {}", e);
                }
            }
            _ => (),
        };
    }
    Ok(())
}
