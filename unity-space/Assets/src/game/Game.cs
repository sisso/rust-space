using System;
using System.Collections;
using System.Collections.Generic;
using System.Diagnostics;
using System.Threading;
using ffi_domain_2;
using game.domain;
using space_data;
using Unity.Collections;
using UnityEngine;
using UnityEngine.Purchasing;
using utils;
using Debug = UnityEngine.Debug;

namespace game
{
    public class Game : MonoBehaviour
    {
        public domain.Domain domain;
        [Range(0.1f, 10.0f)] public float timeDilatation = 1f;
        [Range(1, 10)] public int iterations = 1;
        public double realTime = 0.0;
        public double gameTime = 0.0;
        public List<string> initialArguments;
        public int lastRunTimeMls;
        private Stopwatch stopWatch;
        private ffi_domain_2.SpaceGame game;

        void OnEnable()
        {
            if (this.game == null)
            {
                this.game = new ffi_domain_2.SpaceGame(this.initialArguments);
                this.stopWatch = new Stopwatch();

                this.CreateSectors();
                this.CreateJumps();
                this.UpdateFleets();
            }
        }

        void Destroy()
        {
            if (this.game != null)
            {
                this.game.Dispose();
                this.game = null;
            }
        }

        void CreateSectors()
        {
            var sectors = this.game.GetSectors();
            foreach (var sector in sectors)
            {
                domain.AddSector(sector.GetId(), AsV2D(sector.GetCoords()));
            }
        }

        void CreateJumps()
        {
            var jumps = this.game.GetJumps();
            foreach (var jump in jumps)
            {
                domain.AddJump(jump.GetId(), jump.GetSectorId(), AsV2D(jump.GetCoords()), jump.GetToSectorId(),
                    AsV2D(jump.GetToCoords()));
            }
        }

        private static V2D AsV2D(Tuple<float, float> coords)
        {
            return new V2D(coords.Item1, coords.Item2);
        }

        void UpdateFleets()
        {
            var fleets = this.game.GetFleets();
            foreach (var fleet in fleets)
            {
                var objKind = fleet.GetKind();
                var kind = EntityKind.Fleet;
                switch (objKind)
                {
                    case ffi_domain_2.ObjKind.Asteroid:
                        kind = EntityKind.Asteroid;
                        break;
                    case ffi_domain_2.ObjKind.Station:
                        kind = EntityKind.Station;
                        break;
                }

                domain.AddObj(fleet.GetId(), kind);

                var dockedId = fleet.GetDockedId();
                if (dockedId.IsSome)
                {
                    domain.ObjDock(fleet.GetId(), dockedId.Value);
                }
                else
                {
                    domain.ObjTeleport(fleet.GetId(), fleet.GetSectorId(), AsV2D(fleet.GetCoords()));
                }
            }
        }

        void FixedUpdate()
        {
            this.stopWatch.Reset();
            this.stopWatch.Start();

            var delta = Time.fixedDeltaTime * timeDilatation;
            this.realTime += Time.fixedDeltaTime;
            for (int i = 0; i < iterations; i++)
            {
                this.gameTime += delta;
                this.game.Update(delta);
                // UpdateFleets();

                var events = this.game.TakeEvents();
                foreach (var e in events)
                {
                    var id = e.GetId();
                    var maybeObj = game.GetObj(id);

                    if (!maybeObj.IsSome)
                    {
                        Debug.Log($"fail to finid obj {id}");
                        continue;
                    }

                    var obj = maybeObj.Value;

                    switch (e.GetKind())
                    {
                        case EventKind.Add:
                            var objKind = obj.GetKind();
                            var kind = EntityKind.Fleet;
                            switch (objKind)
                            {
                                case ffi_domain_2.ObjKind.Asteroid:
                                    kind = EntityKind.Asteroid;
                                    break;
                                case ffi_domain_2.ObjKind.Station:
                                    kind = EntityKind.Station;
                                    break;
                            }

                            domain.AddObj(id, kind);
                            break;
                        case EventKind.Move:
                            domain.ObjMove(id, AsV2D(obj.GetCoords()));
                            break;
                        case EventKind.Jump:
                            domain.ObjJump(id, obj.GetSectorId(), AsV2D(obj.GetCoords()));
                            break;
                        case EventKind.Dock:
                            domain.ObjDock(id, obj.GetDockedId().Value);
                            break;
                        case EventKind.Undock:
                            domain.ObjUndock(id, obj.GetSectorId(), AsV2D(obj.GetCoords()));
                            break;
                        default:
                            Debug.Log($"{e.GetKind()} {id}");
                            break;
                    }
                }

                // // get commands from domain to core
                // var requests = this.domain.TakeRequests();
                // this.core.Push(requests);
                // update time
                // this.core.Update(delta);
                // // get all data and send to domain
                // this.core.GetData();
            }

            this.stopWatch.Stop();
            TimeSpan ts = this.stopWatch.Elapsed;
            this.lastRunTimeMls = ts.Milliseconds;
            if (this.lastRunTimeMls > 1)
            {
                Debug.LogWarning($"native update time took {this.lastRunTimeMls}");
            }
        }
    }
}