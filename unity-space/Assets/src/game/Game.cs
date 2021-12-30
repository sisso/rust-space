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
                    domain.ObjTeleport(fleet.GetId(), fleet.GetSectorId(),AsV2D(fleet.GetCoords()));
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
                UpdateFleets();

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